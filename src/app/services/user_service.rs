use std::sync::Arc;
use validator::ValidateRequired;
use crate::app::auth::create_token;
use crate::app::hashing::hash::{hash_password, verify_password};
use crate::app::models::user::{LoginUserPayload, RegisterUserPayload, User};
use crate::app::repositories::user_repository::UserRepository;
use crate::config::config::Config;
use crate::core::error::AppError;
use crate::utils::{generate_password, slugify};

pub struct UserService {
    repo: Arc<dyn UserRepository>
}

impl UserService {
    pub fn new(repo: Arc<dyn UserRepository>) -> UserService {
        Self { repo }
    }

    pub async fn create_user(&self, mut payload: RegisterUserPayload) -> Result<User, AppError> {
        payload.username = slugify(&payload.username);
        payload.email = payload.email.to_lowercase();
        payload.password = match &payload.password.is_empty() {
            true => generate_password(payload.password.len().clone()),
            false => hash_password(&payload.password).unwrap().as_str().to_string()
        };

        if self.repo.find_by_unique_identifier(&payload.username).await?.is_some() {
            return Err(AppError::DuplicateEntry("User already exists".into()));
        }

        if self.repo.find_by_unique_identifier(&payload.email).await?.is_some() {
            return Err(AppError::DuplicateEntry("Email already exists".into()));
        }

        self.repo.create(payload).await
    }

    pub async fn login_handler(
        &self,
        payload: LoginUserPayload,
        config: &Config,
    ) -> Result<String, AppError> {
        let user = self.repo.find_by_unique_identifier(&payload.identifier)
            .await?
            .ok_or_else(|| AppError::InvalidCredentials)?;

        let is_valid = verify_password(&user.password, &payload.password)
            .map_err(|_| AppError::InternalError(String::from("Verify password failed")))?;

        if !is_valid {
            return Err(AppError::InvalidCredentials);
        }

        create_token(user.id, &user.username, config)

    }

    pub async fn find_by_unique_identifier(&self, identifier: &str) -> Result<User, AppError>
    {
        self.repo.find_by_unique_identifier(identifier).await?.ok_or(AppError::NotFound)
    }
}