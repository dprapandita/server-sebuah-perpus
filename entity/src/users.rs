use sea_orm::entity::prelude::*;
use sea_orm::Set;
use chrono::Utc;
use crate::generated::users as generated;

pub use generated::*;

impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, _db: &C, inserted: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        let now = Utc::now();

        if !inserted {
            self.updated_at = Set(now.into())
        }

        Ok(self)
    }
}

