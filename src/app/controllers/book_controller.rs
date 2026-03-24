use crate::app::auth::AuthUser;
use crate::app::models::book::{CreateBookPayload, UpdateBookPayload};
use crate::app::repositories::book_repository::BookFilter;
use crate::app::services::book_service::BookFilterParam;
use crate::app::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use uuid::Uuid;

pub async fn create_book(
    claims: AuthUser,
    State(state): State<AppState>,
    Json(json): Json<CreateBookPayload>,
) -> impl IntoResponse {
    match state.book_service.create_book(json).await {
        Ok(book) => (StatusCode::CREATED, Json(book)).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn update_book(
    claims: AuthUser,
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(json): Json<UpdateBookPayload>,
) -> impl IntoResponse {
    match state.book_service.update_book(&slug, json).await {
        Ok(book) => (StatusCode::OK, Json(book)).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn delete_book(
    claims: AuthUser,
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let book = match state.book_service.get_book_by_slug(&slug).await {
        Ok(b) => b,
        Err(e) => return e.into_response(),
    };

    match state
        .book_service
        .delete_book_with_file_by_id(&book.id.to_string())
        .await
    {
        Ok(()) => (StatusCode::OK, Json("Berhasil dihapus")).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn get_all_books(
    State(state): State<AppState>,
    Query(params): Query<BookFilterParam>,
) -> impl IntoResponse {
    let filter = BookFilter {
        search: params.search,
        status: None,
        limit: params.limit.unwrap_or(10),
        offset: params.offset.unwrap_or(0),
    };
    match state.book_service.get_all_books(filter).await {
        Ok(books) => (StatusCode::OK, Json(books)).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn get_book_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    match state.book_service.find_book_with_file_by_slug(&slug).await {
        Ok(book) => (StatusCode::OK, Json(book)).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn get_book_cover_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let book_id = match Uuid::parse_str(&slug) {
        Ok(id) => id,
        Err(_) => match state.book_service.get_book_by_slug(&slug).await {
            Ok(b) => b.id,
            Err(e) => return e.into_response(),
        },
    };

    match state.book_service.get_book_cover(book_id).await {
        Ok(response) => response.into_response(),
        Err(e) => e.into_response(),
    }
}
