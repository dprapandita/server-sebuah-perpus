use std::time::Duration;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use tracing::log;
use crate::config::config::Config;

pub async fn connect(config: &Config) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(&config.database_url);
    opt.max_connections(10)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(5))
        .acquire_timeout(Duration::from_secs(5))
        .sqlx_logging(true) // logging SQLx debug
        .sqlx_logging_level(log::LevelFilter::Info);

    Database::connect(opt).await
}