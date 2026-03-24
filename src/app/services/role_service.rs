use std::sync::Arc;
use uuid::Uuid;
use crate::app::models::role::{Role, RoleCreatePayload};
use crate::app::repositories::role_repository::RoleRepository;
use crate::core::error::AppError;

pub struct RoleService {
    repo: Arc<dyn RoleRepository>
}

impl RoleService {
    pub fn new(repo: Arc<dyn RoleRepository>) -> Self {
        Self { repo }
    }

    pub async fn create(&self, payload: RoleCreatePayload) -> Result<Role, AppError> {
        self.repo.create(payload).await
    }
    pub async fn get_role_by_name(&self, role_name: &str) -> Result<Role, AppError> {
        self.repo.find_exact_name(role_name).await?.ok_or(AppError::NotFound)
    }
    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repo.delete(id).await
    }

    pub async fn list(&self) -> Result<Vec<Role>, AppError> {
        self.repo.list().await
    }

}