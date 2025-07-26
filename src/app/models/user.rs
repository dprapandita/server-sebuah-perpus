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
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterUserPayload {
    #[validate(length(min = 3, max = 10, message = "name length should be at least 3 (max 10)"))]
    pub username: String,
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