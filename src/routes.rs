use crate::app::controllers::book_controller::{
    create_book, delete_book, get_all_books, get_book_by_slug, get_book_cover_handler, update_book,
};
use crate::app::controllers::book_loan_controller::{
    borrow_book_handler, get_loan_history_handler, return_book_handler,
};
use crate::app::controllers::book_upload_controller::upload_book;
use crate::app::controllers::file_controller::{download_file, serve_file};
use crate::app::controllers::role_controller::{create_role, list_roles};
use crate::app::controllers::user_controller::{create_user, get_user, login_handler};
use crate::app::state::AppState;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

const CONTENT_LENGTH_LIMIT: usize = 50 * 1024 * 1024;

pub fn routes(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", get(|| async { "hello world" }))
        .route("/books", get(get_all_books))
        .route("/book/create", post(create_book))
        .route("/book/upload", post(upload_book))
        .route(
            "/book/{slug}",
            get(get_book_by_slug).put(update_book).delete(delete_book),
        )
        .route("/book/{slug}/cover", get(get_book_cover_handler))
        .route("/roles", get(list_roles).post(create_role))
        .route("/register", post(create_user))
        .route("/login", post(login_handler))
        .route("/me", get(get_user))
        .route("/storage/{*path}", get(serve_file))
        .route("/download/{file_id}", get(download_file))
        .route("/loans/borrow", post(borrow_book_handler))
        .route("/loans/return/{loan_id}", post(return_book_handler))
        .route("/loans/history", get(get_loan_history_handler))
        // Layer
        .layer(TraceLayer::new_for_http())
        .layer(DefaultBodyLimit::max(CONTENT_LENGTH_LIMIT))
        .layer(cors)
        .with_state(state)
}
