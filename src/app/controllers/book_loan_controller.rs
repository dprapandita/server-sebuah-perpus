use crate::app::auth::AuthUser;
use crate::app::models::book_loans::{BookLoanPayloadRequest, BookLoanUpdatePayload};
use crate::app::state::AppState;
use crate::core::error::AppError;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

pub async fn borrow_book_handler(
    State(state): State<AppState>,
    claims: AuthUser,
    Json(payload): Json<BookLoanPayloadRequest>,
) -> impl IntoResponse {
    // Optionally validate that the payload.user_id matches the claims.id
    if payload.user_id != claims.id {
        return AppError::Forbidden.into_response();
    }

    match state.book_loan_service.borrow_book(payload).await {
        Ok(loan) => (StatusCode::CREATED, Json(loan)).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn return_book_handler(
    State(state): State<AppState>,
    claims: AuthUser,
    Path(loan_id): Path<Uuid>,
) -> impl IntoResponse {
    match state
        .book_loan_service
        .return_book(loan_id, claims.id)
        .await
    {
        Ok(loan) => (StatusCode::OK, Json(loan)).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn get_loan_history_handler(
    State(state): State<AppState>,
    claims: AuthUser,
) -> impl IntoResponse {
    match state.book_loan_service.get_loan_history(claims.id).await {
        Ok(histories) => (StatusCode::OK, Json(histories)).into_response(),
        Err(e) => e.into_response(),
    }
}
