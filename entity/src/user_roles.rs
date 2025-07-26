use sea_orm::entity::prelude::*;
use crate::generated::user_roles as generated;

pub use generated::*;

impl ActiveModelBehavior for generated::ActiveModel {}

