use sea_orm::entity::prelude::*;
use sea_orm::Set;
use crate::generated::products as generated;

pub use generated::*;

impl ActiveModelBehavior for generated::ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(Uuid::new_v4()),
            ..ActiveModelTrait::default()
        }
    }
}

