use sea_orm::entity::prelude::*;
use crate::generated::users as generated;

pub use generated::*;

impl ActiveModelBehavior for generated::ActiveModel {}

