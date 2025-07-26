use crate::app::repositories::product_repository::SeaormProductRepository;
use crate::app::services::product_service::ProductService;
use crate::config::config::Config;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use crate::app::repositories::user_repository::SeaormUserRepository;
use crate::app::services::user_service::UserService;

#[derive(Clone)]
pub struct AppState {
    pub database_connection: DatabaseConnection,
    pub env: Config,
    pub product_service: Arc<ProductService>,
    pub user_service: Arc<UserService>
}

impl AppState {
    pub async fn new() -> Self {
        let config = Arc::new(Config::init()).as_ref().clone();
        let db_connection = crate::config::database::connect(&config)
            .await
            .expect("Failed to connect to the database");

        let product_repo = Arc::new(
            SeaormProductRepository::new(Arc::from(db_connection.clone()))
        );
        let product_service = Arc::new(ProductService::new(product_repo));
        let user_repo = Arc::new(
            SeaormUserRepository::new(Arc::from(db_connection.clone()))
        );
        let user_service = Arc::new(UserService::new(user_repo));
        Self {
            database_connection: db_connection,
            env: config,
            product_service,
            user_service
        }
    }
}