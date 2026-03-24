use crate::app::files::epub_parser::EpubMetadata;
use crate::app::files::files::{build_stream_and_response, delete_file, path_storage};
use crate::app::models::book::{Book, BookWithFileResponse, CreateBookPayload, UpdateBookPayload};
use crate::app::models::book_file::{
    BookAggregate, BookFile, BookFileDetail, CreateBookFilePayload,
};
use crate::app::repositories::book_file_repository::BookFileRepository;
use crate::app::repositories::book_repository::{BookFilter, BookRepository};
use crate::core::error::AppError;
use crate::utils::{sanitize_isbn, slugify};
use axum::response::Response;
use sea_orm::{DatabaseConnection, TransactionTrait};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

/// Parameter filter untuk query buku dari controller.
#[derive(Deserialize)]
pub struct BookFilterParam {
    pub search: Option<String>,
    pub status: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

/// Service layer untuk operasi buku dan book_file.
/// Menggunakan `Arc<DatabaseConnection>` agar bisa membuat transaksi per-request.
pub struct BookService {
    db: Arc<DatabaseConnection>,
    repo: Arc<dyn BookRepository>,
    book_file_repo: Arc<dyn BookFileRepository>,
}

impl BookService {
    pub fn new(
        db: Arc<DatabaseConnection>,
        repo: Arc<dyn BookRepository>,
        book_file_repo: Arc<dyn BookFileRepository>,
    ) -> BookService {
        Self {
            db,
            repo,
            book_file_repo,
        }
    }

    // ─── Book Operations ────────────────────────────────────────

    /// Membuat buku baru dalam sebuah transaksi.
    pub async fn create_book(&self, payload: CreateBookPayload) -> Result<Book, AppError> {
        let trx = self.db.begin().await?;
        let result = self.repo.create_with_tx(&trx, payload).await?;
        trx.commit().await?;
        Ok(result)
    }

    /// Mengambil semua buku dengan filter (search, status, limit, offset).
    pub async fn get_all_books(&self, filter: BookFilter) -> Result<Vec<Book>, AppError> {
        self.repo.find_all(filter).await
    }

    /// Mencari buku berdasarkan slug.
    pub async fn get_book_by_slug(&self, slug: &str) -> Result<Book, AppError> {
        self.repo
            .find_by_slug(slug)
            .await?
            .ok_or(AppError::NotFound)
    }

    /// Mengambil cover buku berdasarkan book_id dari agregasi book file.
    pub async fn get_book_cover(&self, book_id: Uuid) -> Result<Response, AppError> {
        let aggregate = self
            .book_file_repo
            .find_aggregate(book_id)
            .await?
            .ok_or(AppError::NotFound)?;

        let cover_path = if let Some(path) = aggregate.book.cover_image {
            path
        } else {
            let cover_file = aggregate
                .book_files
                .iter()
                .find(|f| {
                    matches!(
                        f.file_format.to_lowercase().as_str(),
                        "img" | "jpg" | "jpeg" | "png" | "gif" | "webp"
                    )
                })
                .ok_or(AppError::NotFound)?;
            cover_file.file_path.clone()
        };

        let full_path = path_storage(&cover_path).ok_or(AppError::NotFound)?;

        if !full_path.exists() || !full_path.is_file() {
            return Err(AppError::NotFound);
        }

        let file = tokio::fs::File::open(&full_path)
            .await
            .map_err(|_| AppError::InternalError("System error".to_string()))?;
        let stream = tokio_util::io::ReaderStream::new(file);
        let body = axum::body::Body::from_stream(stream);

        let content_type = match full_path.extension().and_then(|e| e.to_str()) {
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("gif") => "image/gif",
            Some("webp") => "image/webp",
            _ => "application/octet-stream",
        };

        let filename = full_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("cover");

        let response = axum::response::Response::builder()
            .status(axum::http::StatusCode::OK)
            .header(axum::http::header::CONTENT_TYPE, content_type)
            .header(
                axum::http::header::CONTENT_DISPOSITION,
                format!("inline; filename=\"{}\"", filename),
            )
            .body(body)
            .unwrap();

        Ok(response)
    }

    /// Mengupdate buku berdasarkan slug dalam sebuah transaksi.
    pub async fn update_book(
        &self,
        slug: &str,
        payload: UpdateBookPayload,
    ) -> Result<Book, AppError> {
        let trx = self.db.begin().await?;
        let result = self.repo.update(&trx, slug, payload).await?;
        trx.commit().await?;
        Ok(result)
    }

    /// Menghapus buku berdasarkan slug dalam sebuah transaksi.
    pub async fn delete_book(&self, slug: &str) -> Result<(), AppError> {
        let trx = self.db.begin().await?;
        self.repo.delete(&trx, slug).await?;
        trx.commit().await?;
        Ok(())
    }

    // ─── Book + File Combined Operations ──────────────────────────

    /// Membuat buku baru dari metadata EPUB beserta book_file dalam 1 transaksi.
    ///
    /// Flow:
    /// 1. Simpan cover image ke disk (jika ada)
    /// 2. Begin transaction
    /// 3. Insert Book dari metadata EPUB
    /// 4. Insert BookFile dengan info file
    /// 5. Commit transaction
    pub async fn create_book_with_file(
        &self,
        metadata: EpubMetadata,
        file_path: String,
        file_size: i64,
        file_checksum: String,
    ) -> Result<BookWithFileResponse, AppError> {
        // Simpan cover image ke disk jika ada
        let cover_path = if let (Some(ref cover_data), Some(ref cover_mime)) =
            (&metadata.cover_image, &metadata.cover_mime)
        {
            let ext = match cover_mime.as_str() {
                "image/jpeg" => "jpg",
                "image/png" => "png",
                "image/gif" => "gif",
                _ => "img",
            };

            let cover_filename = format!("{}.{}", Uuid::new_v4(), ext);
            let relative_path = format!("uploads/covers/{}", cover_filename);
            let full_path = match path_storage(&relative_path) {
                Some(path) => path,
                None => {
                    return Err(AppError::BadRequest(
                        "Invalid storage path generated".to_owned(),
                    ))
                }
            };

            if let Some(parent) = full_path.parent() {
                tokio::fs::create_dir_all(parent).await.map_err(|e| {
                    AppError::InternalError(format!("Gagal buat direktori cover: {}", e))
                })?;
            }

            tokio::fs::write(&full_path, cover_data)
                .await
                .map_err(|e| AppError::InternalError(format!("Gagal simpan cover: {}", e)))?;

            Some(relative_path)
        } else {
            None
        };

        // Begin transaction
        let trx = self.db.begin().await?;

        // Create book
        let book_payload = CreateBookPayload {
            isbn: sanitize_isbn(Some(metadata.isbn)),
            title: metadata.title.clone(),
            description: metadata.description,
            slug: slugify(&metadata.title),
            author: metadata.author,
            publisher: metadata.publisher,
            cover_image: cover_path,
        };

        let book = match self.repo.create_with_tx(&trx, book_payload).await {
            Ok(b) => b,
            Err(e) => {
                trx.rollback().await.ok();
                tracing::error!("{}", e);
                return Err(e);
            }
        };

        // Create book_file
        let book_file_payload = CreateBookFilePayload {
            book_id: book.id,
            file_path,
            file_format: "epub".to_string(),
            file_size,
            file_checksum,
        };

        let book_file_id = match self
            .book_file_repo
            .create_with_tx(&trx, book_file_payload)
            .await
        {
            Ok(id) => id,
            Err(e) => {
                trx.rollback().await.ok();
                tracing::error!("{}", e);
                return Err(e);
            }
        };

        trx.commit().await?;

        // Ambil book_file yang baru dibuat
        let book_file = self
            .book_file_repo
            .find_by_id(&book_file_id)
            .await?
            .ok_or(AppError::NotFound)?;

        Ok(BookWithFileResponse { book, book_file })
    }

    pub async fn find_book_with_file_by_slug(&self, slug: &str) -> Result<BookAggregate, AppError> {
        let book = self.get_book_by_slug(slug).await?;

        let aggregate = self.get_book_files_by_book_id(book.id).await?;
        Ok(aggregate)
    }

    pub async fn delete_book_with_file_by_id(&self, book_id: &str) -> Result<(), AppError> {
        let book_uuid =
            Uuid::parse_str(book_id).map_err(|e| AppError::BadRequest(e.to_string()))?;

        let aggregate = self.get_book_files_by_book_id(book_uuid).await?;

        let trx = self.db.begin().await?;

        for file in &aggregate.book_files {
            if let Err(e) = self
                .book_file_repo
                .delete(&trx, book_id, &file.file_path)
                .await
            {
                trx.rollback().await.ok();
                return Err(e);
            }
        }

        if let Err(e) = self.repo.delete(&trx, &aggregate.book.slug).await {
            trx.rollback().await.ok();
            return Err(e);
        }

        trx.commit().await?;

        for file in aggregate.book_files {
            delete_file(&file.file_path).await.ok();
        }

        if let Some(cover_path) = aggregate.book.cover_image {
            delete_file(&cover_path).await.ok();
        }

        Ok(())
    }

    // ─── Book File Operations ───────────────────────────────────

    /// Mengambil semua book_file beserta detail buku terkait.
    pub async fn get_all_book_files(&self) -> Result<Vec<BookFileDetail>, AppError> {
        self.book_file_repo.find_all_with_books().await
    }

    /// Mengambil aggregate buku beserta semua file-nya berdasarkan book_id.
    pub async fn get_book_files_by_book_id(
        &self,
        book_id: Uuid,
    ) -> Result<BookAggregate, AppError> {
        self.book_file_repo
            .find_aggregate(book_id)
            .await?
            .ok_or(AppError::NotFound)
    }

    /// Mencari book_file berdasarkan ID.
    pub async fn get_book_file_by_id(&self, id: &str) -> Result<BookFile, AppError> {
        self.book_file_repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)
    }

    /// Membuat book_file baru dalam sebuah transaksi.
    pub async fn create_book_file(
        &self,
        payload: CreateBookFilePayload,
    ) -> Result<String, AppError> {
        let trx = self.db.begin().await?;
        let result = self.book_file_repo.create_with_tx(&trx, payload).await?;
        trx.commit().await?;
        Ok(result)
    }

    /// Menghapus book_file berdasarkan book_id dan file_path dalam sebuah transaksi.
    pub async fn delete_book_file(&self, book_id: &str, file_path: &str) -> Result<(), AppError> {
        let trx = self.db.begin().await?;
        self.book_file_repo.delete(&trx, book_id, file_path).await?;
        trx.commit().await?;
        Ok(())
    }

    pub async fn download_file(&self, file_id: &str) -> Result<Response, AppError> {
        let book_file_uuid =
            Uuid::parse_str(file_id).map_err(|e| AppError::BadRequest(e.to_string()))?;
        let aggregate = self
            .book_file_repo
            .find_aggregate(book_file_uuid)
            .await?
            .ok_or(AppError::NotFound)?;

        let target_file = aggregate.book_files.first().ok_or(AppError::NotFound)?;

        let full_path = path_storage(&target_file.file_path)
            .ok_or(AppError::BadRequest("Path file tidak valid".to_string()))?;

        let extension = &target_file.file_format;
        let custom_name = format!("{}.{}", aggregate.book.title, extension);

        build_stream_and_response(full_path, Some(custom_name))
            .await
            .map_err(|e| AppError::InternalError(format!("Gagal simpan cover: {}", e)))
    }
}
