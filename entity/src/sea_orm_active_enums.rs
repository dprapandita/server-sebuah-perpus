use sea_orm::entity::prelude::*;
use crate::generated::sea_orm_active_enums as generated;

pub use generated::*;

impl ActiveModelBehavior for generated::ActiveModel {}

