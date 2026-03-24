use crate::app::repositories::book_file_repository::SeaOrmBookFileRepository;
use crate::app::repositories::book_loan_repository::SeaORMBookLoanRepository;
use crate::app::repositories::book_repository::SeaOrmBookRepository;
use crate::app::repositories::role_repository::SeaORMRoleRepository;
use crate::app::repositories::user_repository::SeaormUserRepository;
use crate::app::services::book_loan_service::BookLoanService;
use crate::app::services::book_service::BookService;
use crate::app::services::role_service::RoleService;
use crate::app::services::user_service::UserService;
use crate::config::config::Config;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub database_connection: DatabaseConnection,
    pub env: Config,
    pub book_service: Arc<BookService>,
    pub user_service: Arc<UserService>,
    pub role_service: Arc<RoleService>,
    pub book_loan_service: Arc<BookLoanService>,
}

impl AppState {
    pub async fn new() -> Self {
        let config = Arc::new(Config::init()).as_ref().clone();
        let db_connection = crate::config::database::connect(&config)
            .await
            .expect("Failed to connect to the database");

        let db = Arc::from(db_connection.clone());

        let role_repo = Arc::new(SeaORMRoleRepository::new(db.clone()));
        let role_service = Arc::new(RoleService::new(role_repo.clone()));

        let book_repo = Arc::new(SeaOrmBookRepository::new(db.clone()));
        let book_file_repo = Arc::new(SeaOrmBookFileRepository::new(db.clone()));
        let book_service = Arc::new(BookService::new(db.clone(), book_repo.clone(), book_file_repo));

        let user_repo = Arc::new(SeaormUserRepository::new(db.clone()));
        let user_service = Arc::new(UserService::new(db.clone(), user_repo, role_repo));

        let book_loan_repo = Arc::new(SeaORMBookLoanRepository::new(db.clone()));
        let book_loan_service =
            Arc::new(BookLoanService::new(db.clone(), book_loan_repo, book_repo));

        Self {
            database_connection: db_connection,
            env: config,
            book_service,
            user_service,
            role_service,
            book_loan_service,
        }
    }
}
