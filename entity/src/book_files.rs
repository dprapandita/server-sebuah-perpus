use sea_orm::entity::prelude::*;
use crate::generated::book_files as generated;

pub use generated::*;

impl ActiveModelBehavior for generated::ActiveModel {}

