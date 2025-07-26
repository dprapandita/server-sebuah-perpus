use crate::app::controllers::product_controller::{
    create_product, delete_product, get_all_products, get_product_by_slug, update_product
};
use crate::app::state::AppState;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{RequestPartsExt, Router, };
use sea_orm::{ActiveModelTrait};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use crate::app::controllers::user_controller::{create_user, get_user, login_handler};

const CONTENT_LENGTH_LIMIT: usize = 50 * 1024 * 1024;

pub fn routes(state: AppState) -> Router {
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    Router::new()
        .route("/", get(|| async { "hello world" }))
        .route("/products", get(get_all_products))
        .route("/product/create", post(create_product))
        .route(
            "/product/{slug}",
            get(get_product_by_slug).put(update_product).delete(delete_product)
        )
        .route("/register", post(create_user))
        .route("/login", post(login_handler))
        .route("/me", get(get_user))
        // Layer
        .layer(TraceLayer::new_for_http())
        .layer(DefaultBodyLimit::max(CONTENT_LENGTH_LIMIT))
        .layer(cors)
        .with_state(state)
}

pub fn handle_error() -> Router {
    Router::new().fallback(get(not_found_error()))
}

/* Map any error into a `500 Internal Server Error` */
pub fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: ToString,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

// Map str into a `404 Internal Server Error`
pub fn not_found_error() -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, "Not found".to_string())
}
