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

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleInputPayload {
    pub name: String,
}

impl From<entity::roles::Model> for Role {
    fn from(model: entity::roles::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
        }
    }
}