use crate::app::models::book_file::{
    BookAggregate, BookFile, BookFileDetail, CreateBookFilePayload,
};
use crate::core::error::AppError;
use async_trait::async_trait;
use entity::{book_files, books};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    ModelTrait, QueryFilter, Set,
};
use std::sync::Arc;
use uuid::Uuid;

/// Trait yang mendefinisikan operasi-operasi book_file pada database.
#[async_trait]
pub trait BookFileRepository: Send + Sync {
    /// Mengambil semua book_file tanpa detail buku.
    async fn find_all(&self) -> Result<Vec<BookFile>, AppError>;

    /// Mengambil semua book_file beserta detail buku terkait (join).
    async fn find_all_with_books(&self) -> Result<Vec<BookFileDetail>, AppError>;

    /// Membuat book_file baru dalam sebuah transaksi.
    async fn create_with_tx(
        &self,
        trx: &DatabaseTransaction,
        book_file: CreateBookFilePayload,
    ) -> Result<String, AppError>;

    /// Mengambil aggregate buku beserta semua file-nya (1:N).
    async fn find_aggregate(&self, book_id: Uuid) -> Result<Option<BookAggregate>, AppError>;

    /// Mencari book_file berdasarkan ID.
    async fn find_by_id(&self, id: &str) -> Result<Option<BookFile>, AppError>;

    /// Menghapus book_file berdasarkan book_id dan file_path dalam sebuah transaksi.
    async fn delete(
        &self,
        trx: &DatabaseTransaction,
        book_id: &str,
        file_path: &str,
    ) -> Result<(), AppError>;
}

pub struct SeaOrmBookFileRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmBookFileRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> SeaOrmBookFileRepository {
        SeaOrmBookFileRepository { db }
    }
}

#[async_trait]
impl BookFileRepository for SeaOrmBookFileRepository {
    async fn find_all(&self) -> Result<Vec<BookFile>, AppError> {
        let result = book_files::Entity::find().all(self.db.as_ref()).await?;
        Ok(result.into_iter().map(|b| b.into()).collect())
    }

    async fn find_all_with_books(&self) -> Result<Vec<BookFileDetail>, AppError> {
        let results = book_files::Entity::find()
            .find_also_related(books::Entity)
            .all(self.db.as_ref())
            .await?;

        let details = results
            .into_iter()
            .filter_map(|(file, book_opt)| {
                book_opt.map(|book| BookFileDetail {
                    id: file.id,
                    book_id: file.book_id,
                    book_title: book.title,
                    book_slug: book.slug,
                    cover_image: book.cover_image,
                    file_path: file.file_path,
                    file_format: file.file_format,
                    file_size: file.file_size,
                    file_checksum: file.checksum,
                })
            })
            .collect();

        Ok(details)
    }

    async fn create_with_tx(
        &self,
        trx: &DatabaseTransaction,
        book_file: CreateBookFilePayload,
    ) -> Result<String, AppError> {
        let new_book_file = book_files::ActiveModel {
            id: Set(Uuid::new_v4()),
            book_id: Set(book_file.book_id),
            file_size: Set(book_file.file_size),
            file_path: Set(book_file.file_path),
            file_format: Set(book_file.file_format),
            checksum: Set(book_file.file_checksum),
            ..Default::default()
        };
        let result = new_book_file.insert(trx).await?;
        Ok(result.id.to_string())
    }

    async fn find_aggregate(&self, book_id: Uuid) -> Result<Option<BookAggregate>, AppError> {
        let book = books::Entity::find_by_id(book_id)
            .one(self.db.as_ref())
            .await?;

        match book {
            Some(book_model) => {
                let files = book_model
                    .find_related(book_files::Entity)
                    .all(self.db.as_ref())
                    .await?;
                Ok(Some(BookAggregate {
                    book: book_model,
                    book_files: files,
                }))
            }
            None => Ok(None),
        }
    }

    async fn find_by_id(&self, book_id: &str) -> Result<Option<BookFile>, AppError> {
        let uuid = Uuid::parse_str(book_id)
            .map_err(|_| AppError::Validation("ID tidak valid".to_string()))?;

        let result = book_files::Entity::find()
            .filter(book_files::Column::Id.eq(uuid))
            .one(self.db.as_ref())
            .await?;

        Ok(result.map(|f| f.into()))
    }

    async fn delete(
        &self,
        trx: &DatabaseTransaction,
        book_id: &str,
        file_path: &str,
    ) -> Result<(), AppError> {
        let book_id = Uuid::parse_str(book_id)
            .map_err(|_| AppError::Validation("book_id tidak valid".to_string()))?;

        let result = book_files::Entity::delete_many()
            .filter(book_files::Column::BookId.eq(book_id))
            .filter(book_files::Column::FilePath.eq(file_path))
            .exec(trx)
            .await?;

        if result.rows_affected == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }
}
