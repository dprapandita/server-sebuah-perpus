use axum::extract::{Path, State};
use axum::Json;
use axum::response::IntoResponse;
use uuid::Uuid;
use crate::app::auth::AuthUser;
use crate::app::models::role::RoleCreatePayload;
use crate::app::models::user::AssignRole;
use crate::app::state::AppState;
use crate::core::error::AppError;

pub async fn list_roles(
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // require_roles(&user, &["admin"])?;
    match state.role_service.list().await {
        Ok(roles) => Ok(Json(roles).into_response()),
        Err(e) => Err(e),
    }
}

pub async fn create_role(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<RoleCreatePayload>,
) -> Result<impl IntoResponse, AppError> {
    // require_roles(&user, &["admin"])?;
    match state.role_service.create(payload).await {
        Ok(role) => Ok(Json(role).into_response()),
        Err(e) => Err(e),
    }
}

pub async fn attach_to_user(
    user: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<AssignRole>,
) -> Result<impl IntoResponse, AppError> {
    // require_roles(&user, &["admin"])?;
    match state
        .user_service
        .attach_role(user_id, payload.role_id)
        .await
    {
        Ok(role) => Ok(Json(role).into_response()),
        Err(e) => Err(e),
    }
}