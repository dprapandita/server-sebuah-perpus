use crate::app::models::user::{RegisterUserPayload, User};
use crate::core::error::AppError;
use async_trait::async_trait;
use entity::generated::prelude::Users;
use entity::users;
use entity::users::Model;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, QueryFilter, RuntimeErr, Set};
use std::sync::Arc;
use uuid::Uuid;

fn model_to_entity(model: Model) -> User {
    User {
        id: model.id,
        name: model.name,
        username: model.username,
        email: model.email,
        password: model.password,
    }
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, payload: RegisterUserPayload) -> Result<User, AppError>;
    async fn find_by_unique_identifier(&self, identifier: &str) -> Result<Option<User>, AppError>;
}

pub struct SeaormUserRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaormUserRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for SeaormUserRepository {
    async fn create(&self, payload: RegisterUserPayload) -> Result<User, AppError> {
        let new_user = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(payload.name),
            username: Set(payload.username),
            email: Set(payload.email),
            password: Set(payload.password),
            ..Default::default()
        };
        match new_user.insert(self.db.as_ref()).await {
            Ok(model ) => Ok(model_to_entity(model)),
            Err(db_err) => {
                // Periksa apakah ini error dari database
                if let DbErr::Query(RuntimeErr::SqlxError(sqlx_error)) = db_err {
                    // Periksa apakah error ini adalah error dari database
                    if let Some(db_err_info) = sqlx_error.as_database_error() {
                        // Dapatkan kode error spesifik (contoh: "23505" untuk unique violation di Postgres)
                        if db_err_info.code().as_deref() == Some("23505") {
                            return Err(AppError::DuplicateEntry(
                                "Email atau username sudah terdaftar.".to_string(),
                            ));
                        }
                    }
                }
                // Jika bukan error unique, kembalikan sebagai error database biasa
                Err(AppError::InternalError("Gagal membuat pengguna.".to_string()))
            }
        }

    }

    async fn find_by_unique_identifier(&self, identifier: &str) -> Result<Option<User>, AppError> {
        let is_uuid = Uuid::parse_str(identifier).is_ok();

        let model = Users::find()
            .filter(
                Condition::any()
                    .add(if is_uuid {
                        users::Column::Id.eq(identifier)
                    } else {
                        // Kalo bukan uuid, maka gak cocok
                        users::Column::Id.eq(Uuid::nil())
                    })
                    .add(users::Column::Username.eq(identifier))
                    .add(users::Column::Email.eq(identifier)),
            )
            .one(self.db.as_ref())
            .await?;
        Ok(model.map(model_to_entity))
    }
}