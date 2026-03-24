use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use tracing::info;

use crate::app::auth::AuthUser;
use crate::app::files::epub_parser::parse_epub;
use crate::app::files::files::{path_storage, write_file_direct};
use crate::app::files::validator::{validate_book_mime, validate_chunk_size, MAX_UPLOAD_SIZE};
use crate::app::state::AppState;
use crate::core::error::AppError;

/// Handler untuk upload file EPUB dan membuat buku baru dalam 1 flow.
///
/// Endpoint: `POST /book/upload`
///
/// Multipart fields:
/// - `file`: File EPUB yang akan di-upload
///
/// Flow:
/// 1. Terima file EPUB via multipart
/// 2. Validasi MIME type & ukuran
/// 3. Simpan file ke storage via `write_file_direct()`
/// 4. Parse EPUB → extract metadata
/// 5. Panggil `book_service.create_book_with_file()` (1 transaksi)
/// 6. Return `BookWithFileResponse`
pub async fn upload_book(
    claims: AuthUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    // 1. Ambil field "file" dari multipart dengan iterasi yang aman
    let mut original_filename_opt = None;
    let mut file_bytes_opt = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("file") {
            if let Some(name) = field.file_name() {
                if !name.is_empty() {
                    original_filename_opt = Some(name.to_string());
                }
            }

            let bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::InternalError(format!("Gagal baca file: {}", e)))?;

            file_bytes_opt = Some(bytes);
            break; // Stop jika field "file" sudah ditemukan
        }
    }

    let file_bytes = file_bytes_opt.ok_or(AppError::FileNotFound)?;

    let original_filename = original_filename_opt.ok_or_else(|| {
        AppError::Validation(
            "File yang di-upload harus menyertakan nama file (filename)".to_string(),
        )
    })?;

    // Validasi MIME type
    if let Err(e) = validate_book_mime(&file_bytes) {
        return Err(AppError::Validation(format!("File tidak valid: {}", e)));
    }

    // Validasi ukuran file (misal maksimal 50MB menggunakan konstanta yang ada)
    if let Err(e) = validate_chunk_size(&file_bytes, MAX_UPLOAD_SIZE / 1024 / 1024) {
        return Err(AppError::Validation(format!(
            "Ukuran file terlalu besar: {}",
            e
        )));
    }

    // Simpan file ke storage via write_file_direct
    let (relative_path, file_checksum, file_size) =
        write_file_direct(&original_filename, &file_bytes, "books")
            .await
            .map_err(|(_status, msg)| AppError::InternalError(msg))?;

    info!("File EPUB tersimpan: {}", relative_path);

    // Helper closure untuk membersihkan file dan hash jika proses selanjutnya gagal
    let full_path = match path_storage(&relative_path) {
        Some(path) => path,
        None => return Err(AppError::BadRequest("Invalid storage path generated".to_owned())),
    };
    let cleanup_files = |path: &std::path::Path| {
        let path_clone = path.to_path_buf();
        tokio::spawn(async move {
            tokio::fs::remove_file(&path_clone).await.ok();
            tokio::fs::remove_file(path_clone.with_extension("hash"))
                .await
                .ok();
        });
    };

    // Parse EPUB metadata
    let full_path_str = full_path.to_string_lossy().to_string();
    let metadata = match parse_epub(&full_path_str) {
        Ok(m) => m,
        Err(e) => {
            cleanup_files(&full_path);
            return Err(e);
        }
    };

    info!(
        "EPUB metadata: title={}, author={}",
        metadata.title, metadata.author
    );

    // Buat Book + BookFile dalam 1 transaksi
    match state
        .book_service
        .create_book_with_file(metadata, relative_path, file_size, file_checksum)
        .await
    {
        Ok(response) => Ok((StatusCode::CREATED, Json(response))),
        Err(e) => {
            cleanup_files(&full_path);
            Err(e)
        }
    }
}
