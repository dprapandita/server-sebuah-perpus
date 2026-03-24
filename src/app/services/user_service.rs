use crate::app::auth::create_token;
use crate::app::hashing::hash::{hash_password, verify_password};
use crate::app::models::user::{
    LoginResponse, LoginUserPayload, RegisterUserPayload, User, UserInfo, UserRegisterValue,
};
use crate::app::repositories::user_repository::UserRepository;
use crate::config::config::Config;
use crate::core::error::AppError;
use crate::utils::{generate_password, generate_username};
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;
use uuid::Uuid;
use crate::app::repositories::role_repository::RoleRepository;

pub struct UserService {
    db: Arc<DatabaseConnection>,
    repo: Arc<dyn UserRepository>,
    role_repo: Arc<dyn RoleRepository>,
}

impl UserService {
    pub fn new(
        db: Arc<DatabaseConnection>,
        repo: Arc<dyn UserRepository>,
        role_repo: Arc<dyn RoleRepository>,
    ) -> UserService {
        Self { db, repo, role_repo }
    }

    pub async fn create_user(&self, mut payload: RegisterUserPayload) -> Result<User, AppError> {
        let trx = self.db.begin().await?;

        let final_username = match payload.username {
            Some(val) => val,
            None => generate_username()?,
        };
        payload.email = payload.email.to_lowercase();
        payload.password = if payload.password.is_empty() {
            generate_password(16)
        } else {
            hash_password(&payload.password).map_err(|e| AppError::InternalError(e.to_string()))?
        };

        if let Some(exist) = self
            .repo
            .find_conflict_email_or_username(&trx, &final_username, &payload.email)
            .await?
        {
            if exist.username == final_username {
                return Err(AppError::DuplicateEntry("Username already exists".into()));
            }
            if exist.email == payload.email {
                return Err(AppError::DuplicateEntry("Email already exists".into()));
            }
            // fallback (kalau ada field lain)
            return Err(AppError::DuplicateEntry("User already exists".into()));
        }
        let user_value = UserRegisterValue {
            username: final_username,
            name: payload.name,
            password: payload.password,
            email: payload.email,
        };

        let mut result = self.repo.create_with_tx(&trx, &user_value).await?;

        let role = self.role_repo.find_exact_name("member")
            .await?
            .ok_or_else(|| AppError::NotFound)?;

        self.repo.attach_role_tx(&trx, result.id, role.id).await?;

        result.roles = vec![role.name];

        trx.commit().await?;

        Ok(result)
    }

    pub async fn login_handler(
        &self,
        payload: LoginUserPayload,
        config: &Config,
    ) -> Result<LoginResponse, AppError> {
        let user = self
            .repo
            .find_by_unique_identifier(&payload.identifier)
            .await?
            .ok_or_else(|| AppError::InvalidCredentials)?;

        let is_valid = verify_password(&user.password, &payload.password)
            .map_err(|_| AppError::InternalError(String::from("Verify password failed")))?;

        if !is_valid {
            return Err(AppError::InvalidCredentials);
        }

        let token = create_token(user.id, &user.username, user.roles.clone(), config)
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        let user_info = UserInfo {
            id: user.id,
            full_name: user.name,
            username: user.username,
            email: user.email,
            roles: user.roles,
        };
        Ok(LoginResponse {
            token,
            user: user_info,
        })
    }

    pub async fn find_by_unique_identifier(&self, identifier: &str) -> Result<User, AppError> {
        self.repo
            .find_by_unique_identifier(identifier)
            .await?
            .ok_or(AppError::NotFound)
    }

    pub async fn attach_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), AppError> {
        self.repo.attach_role(user_id, role_id).await
    }
}
