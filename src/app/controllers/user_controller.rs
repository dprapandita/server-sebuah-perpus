use crate::app::auth::AuthUser;
use crate::app::models::user::{LoginUserPayload, RegisterUserPayload};
use crate::app::state::AppState;
use crate::core::error::AppError;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use validator::Validate;

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUserPayload>,
) -> impl IntoResponse {
    if let Err(e) = payload.validate() {
        return AppError::Validation(e.to_string()).into_response();
    }

    match state.user_service.create_user(payload).await {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginUserPayload>,
) -> impl IntoResponse {
    let config = state.env.clone();

    match state.user_service.login_handler(payload, &config).await {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn get_user(
    AuthUser(claims): AuthUser,
) -> impl IntoResponse {
    (StatusCode::OK, Json(claims))
}