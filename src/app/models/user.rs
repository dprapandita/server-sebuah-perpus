use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterUserPayload {
    #[validate(length(min = 3, max = 10, message = "name length should be at least 3 (max 10)"))]
    pub username: Option<String>,
    #[validate(length(min = 3, message = "Nama minimal 3 karakter."))]
    pub name: String,
    #[validate(length(min = 8, message = "Password minimal 8 karakter."))]
    pub password: String,
    // #[validate(email(message = "Format email tidak valid."))]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginUserPayload {
    pub identifier: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateUserPayload {
    #[validate(length(
        min = 3,
        max = 10,
        message = "name length should be at least 3 (max 10)"
    ))]
    pub username: Option<String>,
    #[validate(length(min = 3, message = "Nama minimal 3 karakter."))]
    pub name: Option<String>,
    #[validate(length(min = 8, message = "Password minimal 8 karakter."))]
    pub password: Option<String>,
    // #[validate(email(message = "Format email tidak valid."))]
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssignRole {
    pub role_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub id: Uuid,
    pub full_name: String,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UserRegisterValue {
    pub username: String,
    pub name: String,
    pub email: String,
    #[validate(length(min = 8, message = "Password minimal 8 karakter."))]
    pub password: String
}

impl From<(entity::users::Model, Vec<entity::roles::Model>)> for User {
    fn from((user_model, role_model): (entity::users::Model, Vec<entity::roles::Model>)) -> Self {
        Self {
            id: user_model.id,
            name: user_model.name,
            username: user_model.username,
            email: user_model.email,
            password: user_model.password,
            roles: role_model.into_iter().map(|role| role.name).collect(),
        }
    }
}