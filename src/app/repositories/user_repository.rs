use crate::app::models::user::{UpdateUserPayload, User, UserRegisterValue};
use crate::core::error::AppError;
use async_trait::async_trait;
use entity::users;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait, QueryFilter, RuntimeErr, Set};
use std::sync::Arc;
use tracing::log::info;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_with_tx(
        &self,
        tx: &DatabaseTransaction,
        payload: &UserRegisterValue,
    ) -> Result<User, AppError>;
    async fn find_by_unique_identifier(&self, identifier: &str) -> Result<Option<User>, AppError>;
    async fn find_conflict_email_or_username(
        &self,
        tx: &DatabaseTransaction,
        username: &str,
        email: &str,
    ) -> Result<Option<User>, AppError>;
    async fn list_all(&self) -> Result<Vec<User>, AppError>;
    async fn update_user(
        &self,
        identifier: &str,
        payload: UpdateUserPayload,
    ) -> Result<User, AppError>;
    async fn attach_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), AppError>;
    async fn attach_role_tx(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), AppError>;
    async fn delete_user_by_id(&self, user_id: &str) -> Result<(), AppError>;
}

pub struct SeaormUserRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaormUserRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn identifier_condition(identifier: &str) -> Condition {
        let mut cond = Condition::any()
            .add(users::Column::Username.eq(identifier))
            .add(users::Column::Email.eq(identifier));

        if let Ok(id) = Uuid::parse_str(identifier) {
            cond = cond.add(users::Column::Id.eq(id));
        }

        cond
    }

    async fn find_model_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<Option<users::Model>, AppError> {
        let user = users::Entity::find()
            .filter(Condition::all().add(Self::identifier_condition(identifier)))
            .one(self.db.as_ref())
            .await?;
        Ok(user)
    }

    async fn find_user_with_roles(&self, identifier: &str) -> Result<Option<User>, AppError> {
        let user_with_roles = users::Entity::find()
            .filter(Condition::all().add(Self::identifier_condition(identifier)))
            .find_with_related(entity::generated::prelude::Roles)
            .all(self.db.as_ref())
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?
            .into_iter()
            .next();

        Ok(user_with_roles
            .into_iter()
            .next()
            .map(|(user_model, role_models)| (user_model, role_models).into()))
    }

    async fn find_all_with_roles(&self) -> Result<Vec<User>, AppError> {
        let users_with_roles = users::Entity::find()
            .find_with_related(entity::generated::prelude::Roles)
            .all(self.db.as_ref())
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        Ok(users_with_roles
            .into_iter()
            .map(|(user_model, role_models)| (user_model, role_models).into())
            .collect())
    }
}

#[async_trait]
impl UserRepository for SeaormUserRepository {
    async fn create_with_tx(
        &self,
        tx: &DatabaseTransaction,
        payload: &UserRegisterValue,
    ) -> Result<User, AppError> {
        let new_user = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(payload.name.clone()),
            username: Set(payload.username.clone()),
            email: Set(payload.email.clone()),
            password: Set(payload.password.clone()),
            ..Default::default()
        };
        info!("Creating user");
        match new_user.insert(tx).await {
            Ok(model) => {
                Ok(User {
                    id: model.id,
                    name: model.name,
                    username: model.username,
                    email: model.email,
                    password: model.password,
                    roles: vec![],
                })
            }
            Err(db_err) => {
                if let DbErr::Query(RuntimeErr::SqlxError(sqlx_error)) = db_err {
                    if let Some(db_err_info) = sqlx_error.as_database_error() {
                        if db_err_info.code().as_deref() == Some("23505") {
                            return Err(AppError::DuplicateEntry(
                                "Email atau username sudah terdaftar.".to_string(),
                            ));
                        }
                    }
                }
                Err(AppError::InternalError(
                    "Gagal membuat pengguna.".to_string(),
                ))
            }
        }
    }

    async fn find_by_unique_identifier(&self, identifier: &str) -> Result<Option<User>, AppError> {
        self.find_user_with_roles(identifier).await
    }

    async fn find_conflict_email_or_username(
        &self,
        tx: &DatabaseTransaction,
        username: &str,
        email: &str,
    ) -> Result<Option<User>, AppError> {
        let conflict = users::Entity::find()
            .filter(
                users::Column::Username
                    .eq(username)
                    .or(users::Column::Email.eq(email)),
            )
            .one(tx)
            .await?;
        if let Some(user_model) = conflict {
            // Ambil user dengan roles, fallback ke empty roles jika belum ada
            let user = self
                .find_user_with_roles(&user_model.username)
                .await?
                .unwrap_or_else(|| User {
                    id: user_model.id,
                    name: user_model.name,
                    username: user_model.username,
                    email: user_model.email,
                    password: user_model.password,
                    roles: vec![],
                });

            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    async fn list_all(&self) -> Result<Vec<User>, AppError> {
        self.find_all_with_roles().await
    }

    async fn update_user(
        &self,
        identifier: &str,
        payload: UpdateUserPayload,
    ) -> Result<User, AppError> {
        let model = self
            .find_model_by_identifier(identifier)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        match model {
            Some(model) => {
                let user_id = model.id;
                let mut active_model: users::ActiveModel = model.into();
                if let Some(name) = payload.name {
                    active_model.name = Set(name.to_string());
                }
                if let Some(username) = payload.username {
                    active_model.username = Set(username.to_string());
                }
                if let Some(email) = payload.email {
                    active_model.email = Set(email.to_string());
                }
                if let Some(password) = payload.password {
                    active_model.password = Set(password.to_string());
                }
                active_model
                    .update(self.db.as_ref())
                    .await
                    .map_err(|e| AppError::InternalError(e.to_string()))?;

                // Fetch updated user with roles
                self.find_user_with_roles(&user_id.to_string())
                    .await?
                    .ok_or(AppError::NotFound)
            }
            None => Err(AppError::NotFound),
        }
    }

    async fn attach_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), AppError> {
        let existing = users::Entity::find()
            .filter(users::Column::Id.eq(user_id))
            .one(self.db.as_ref())
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if existing.is_none() {
            return Err(AppError::NotFound);
        }

        let new_role = entity::user_roles::ActiveModel {
            user_id: Set(existing.unwrap().id),
            role_id: Set(role_id),
            ..Default::default()
        };
        new_role.insert(self.db.as_ref()).await?;
        Ok(())
    }

    async fn attach_role_tx(
        &self,
        tx: &DatabaseTransaction,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), AppError> {
        let existing = users::Entity::find()
            .filter(users::Column::Id.eq(user_id))
            .one(tx)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        if existing.is_none() {
            return Err(AppError::NotFound);
        }

        let new_role = entity::user_roles::ActiveModel {
            user_id: Set(existing.unwrap().id),
            role_id: Set(role_id),
            ..Default::default()
        };
        new_role.insert(tx).await?;
        Ok(())
    }

    async fn delete_user_by_id(&self, user_id: &str) -> Result<(), AppError> {
        let parse_uuid =
            Uuid::parse_str(user_id).map_err(|e| AppError::InternalError(e.to_string()))?;
        let model = users::Entity::delete_by_id(parse_uuid)
            .exec(self.db.as_ref())
            .await;
        match model {
            Ok(_) => Ok(()),
            Err(db_err) => Err(AppError::InternalError(db_err.to_string())),
        }
    }
}
