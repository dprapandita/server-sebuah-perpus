use crate::app::files::files::path_storage;
use crate::app::state::AppState;
use crate::core::error::AppError;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use tokio_util::io::ReaderStream;

/// Handler untuk serve file dari storage.
///
/// Endpoint: `GET /storage/{*path}`
///
/// Resolve path menggunakan `path_storage()` lalu kirim file sebagai response.
pub async fn serve_file(Path(path): Path<String>) -> Result<Response, AppError> {
    // 1. Resolve path menggunakan Option untuk keamanan
    let full_path = match path_storage(&path) {
        Some(p) => p,
        None => return Ok(AppError::NotFound.into_response()),
    };

    // 2. Cek apakah file fisik ada di harddisk
    if !full_path.exists() || !full_path.is_file() {
        return Ok(AppError::NotFound.into_response());
    }

    // 3. Buka file secara asynchronous (GAK PAKAI tokio::fs::read LAGI!)
    let file = match tokio::fs::File::open(&full_path).await {
        Ok(file) => file,
        Err(_) => return Err(AppError::InternalError("System error".to_string())),
    };

    // 4. Ubah file menjadi aliran data (Stream)
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // 5. Tentukan content type berdasarkan extension
    let content_type = match full_path.extension().and_then(|e| e.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    };

    // 6. Ambil nama file untuk Content-Disposition
    let filename = full_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");

    // 7. Kirim Response dengan header INLINE
    Ok((
        [
            (header::CONTENT_TYPE, content_type.to_string()),
            (
                header::CONTENT_DISPOSITION,
                // Menggunakan 'inline' agar gambar dirender di aplikasi/browser
                format!("inline; filename=\"{}\"", filename),
            ),
        ],
        body,
    )
        .into_response())
}

pub async fn download_file(
    State(state): State<AppState>,
    Path(file_id): Path<String>,
) -> Result<Response, AppError> {
    match state.book_service.download_file(&file_id).await {
        Ok(response) => Ok(response),
        Err(e) => Err(AppError::InternalError(format!("{}", e))),
    }
}
