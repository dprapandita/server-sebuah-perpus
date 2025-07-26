use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Role {
    pub id: uuid::Uuid,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleCreatePayload {
    pub name: String,
}