use crate::app::models::role::{Role, RoleCreatePayload};
use crate::core::error::AppError;
use async_trait::async_trait;
use entity::generated::prelude::Roles;
use entity::generated::roles;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;
use uuid::Uuid;

#[async_trait]
pub trait RoleRepository: Send + Sync {
    async fn create(&self, payload: RoleCreatePayload) -> Result<Role, AppError>;
    async fn find_exact_name(&self, role_name: &str) -> Result<Option<Role>, AppError>;
    async fn find_by_id(&self, role_id: Uuid) -> Result<Option<Role>, AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
    async fn list(&self) -> Result<Vec<Role>, AppError>;
}

pub struct SeaORMRoleRepository {
    db: Arc<DatabaseConnection>
}

impl SeaORMRoleRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> SeaORMRoleRepository {
        SeaORMRoleRepository { db }
    }
}

#[async_trait]
impl RoleRepository for SeaORMRoleRepository {
    async fn create(&self, payload: RoleCreatePayload) -> Result<Role, AppError> {
        let new_role = roles::ActiveModel {
            name: Set(payload.name.to_owned()),
            ..Default::default()
        };
        match new_role.insert(self.db.as_ref()).await {
            Ok(model) => Ok(model.into()),
            Err(db_error) => {
                Err(AppError::from(db_error))
            }
        }
    }

    async fn find_exact_name(&self, role_name: &str) -> Result<Option<Role>, AppError> {
        let exist = roles::Entity::find()
            .filter(roles::Column::Name.eq(role_name))
            .one(self.db.as_ref())
            .await?
            .ok_or(AppError::NotFound)?;
        Ok(Some(exist.into()))
    }

    async fn find_by_id(&self, role_id: Uuid) -> Result<Option<Role>, AppError> {
        let exist = roles::Entity::find()
            .filter(roles::Column::Id.eq(role_id))
            .one(self.db.as_ref())
            .await?
            .ok_or(AppError::NotFound)?;
        Ok(Some(exist.into()))
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let model = Roles::delete_many()
            .filter(roles::Column::Id.eq(id))
            .exec(self.db.as_ref())
            .await?;
        if model.rows_affected == 0 {
            Err(AppError::NotFound)
        }
        else {
            Ok(())
        }
    }

    async fn list(&self) -> Result<Vec<Role>, AppError> {
        let models = Roles::find().all(self.db.as_ref()).await?;
        let roles = models.into_iter().map(Into::into).collect();
        Ok(roles)
    }
}