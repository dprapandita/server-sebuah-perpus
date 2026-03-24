use crate::app::models::book::{Book, CreateBookPayload, UpdateBookPayload};
use crate::core::error::AppError;
use crate::utils::slugify;
use async_trait::async_trait;
use entity::books;
use entity::generated::prelude::Books;
use entity::generated::sea_orm_active_enums::{BookStatusLoanType, BookStatusType};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DatabaseTransaction, EntityTrait,
    IntoActiveModel, QueryFilter, QuerySelect, Set,
};
use std::sync::Arc;
use uuid::Uuid;

pub struct BookFilter {
    pub search: Option<String>,
    pub status: Option<BookStatusType>,
    pub limit: u64,
    pub offset: u64,
}

impl Default for BookFilter {
    fn default() -> Self {
        Self {
            search: None,
            status: None,
            limit: 10,
            offset: 0,
        }
    }
}

fn parse_book_status(book_status: &str) -> Result<BookStatusType, AppError> {
    match book_status.trim() {
        "Published" | "Dipublish" => Ok(BookStatusType::Published),
        "Archived" | "Diarsip" => Ok(BookStatusType::Archived),
        "Draft" => Ok(BookStatusType::Draft),
        "Preview" | "Direview" => Ok(BookStatusType::Previewing),
        "Takedown" | "Dihapus" => Ok(BookStatusType::Takedown),
        _ => Err(AppError::Validation("Gada lagi".to_string())),
    }
}

#[async_trait]
pub trait BookRepository: Send + Sync {
    async fn create_with_tx(
        &self,
        trx: &DatabaseTransaction,
        payload: CreateBookPayload,
    ) -> Result<Book, AppError>;
    async fn find_all(&self, filter: BookFilter) -> Result<Vec<Book>, AppError>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Book>, AppError>;
    async fn find_by_id(&self, book_id: Uuid) -> Result<Option<Book>, AppError>;
    async fn update(
        &self,
        trx: &DatabaseTransaction,
        slug: &str,
        payload: UpdateBookPayload,
    ) -> Result<Book, AppError>;
    async fn update_status_loan(
        &self,
        trx: &DatabaseTransaction,
        book_id: Uuid,
        status_loan: Option<entity::generated::sea_orm_active_enums::BookStatusLoanType>,
    ) -> Result<(), AppError>;
    async fn delete(&self, trx: &DatabaseTransaction, slug: &str) -> Result<(), AppError>;
}

pub struct SeaOrmBookRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmBookRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn apply_filter(
        mut q: sea_orm::Select<books::Entity>,
        filter: &BookFilter,
    ) -> sea_orm::Select<books::Entity> {
        if let Some(ref search) = filter.search {
            let search = format!("%{}%", &search.trim());
            q = q.filter(
                Condition::any()
                    .add(books::Column::Title.eq(&search))
                    .add(books::Column::Publisher.eq(&search)),
            );
        }

        if let Some(ref status) = filter.status {
            q = q.filter(books::Column::Status.eq(status.clone()));
        }

        q.limit(filter.limit).offset(filter.offset)
    }
}

#[async_trait]
impl BookRepository for SeaOrmBookRepository {
    async fn create_with_tx(
        &self,
        trx: &DatabaseTransaction,
        payload: CreateBookPayload,
    ) -> Result<Book, AppError> {
        let new_product = books::ActiveModel {
            id: Set(Uuid::new_v4()),
            isbn: Set(payload.isbn),
            title: Set(payload.title.to_string()),
            slug: Set(slugify(payload.title.as_str())),
            description: Set(payload.description.to_string()),
            publisher: Set(payload.publisher.to_owned()),
            author: Set(payload.author.to_owned()),
            cover_image: Set(payload.cover_image),
            status: Set(Some(BookStatusType::Published)),
            status_loan: Set(Some(BookStatusLoanType::Active)),
            ..Default::default()
        };

        let result = new_product.insert(trx).await?;
        Ok(result.into())
    }

    async fn find_all(&self, filter: BookFilter) -> Result<Vec<Book>, AppError> {
        let models = Books::find();
        let books = Self::apply_filter(models, &filter)
            .offset(filter.offset)
            .limit(filter.limit)
            .all(self.db.as_ref())
            .await?
            .into_iter()
            .map(Into::into)
            .collect();
        Ok(books)
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Book>, AppError> {
        let model = books::Entity::find()
            .filter(books::Column::Slug.eq(slug))
            .one(self.db.as_ref())
            .await?;
        Ok(model.map(Into::into))
    }

    async fn find_by_id(&self, book_id: Uuid) -> Result<Option<Book>, AppError> {
        let model = books::Entity::find_by_id(book_id)
            .one(self.db.as_ref())
            .await?;
        Ok(model.map(Into::into))
    }

    async fn update(
        &self,
        trx: &DatabaseTransaction,
        slug: &str,
        payload: UpdateBookPayload,
    ) -> Result<Book, AppError> {
        let model = Books::find()
            .filter(books::Column::Slug.eq(slug))
            .one(trx)
            .await?
            .ok_or(AppError::NotFound)?;
        let mut active_model = model.into_active_model();
        active_model.title = Set(payload.title.to_string());
        active_model.description = Set(payload.description.to_string());
        active_model.publisher = Set(payload.publisher);
        active_model.author = Set(payload.author);
        let result = active_model.update(trx).await?;

        Ok(result.into())
    }

    async fn update_status_loan(
        &self,
        trx: &DatabaseTransaction,
        book_id: Uuid,
        status_loan: Option<entity::generated::sea_orm_active_enums::BookStatusLoanType>,
    ) -> Result<(), AppError> {
        let model = Books::find_by_id(book_id)
            .one(trx)
            .await?
            .ok_or(AppError::NotFound)?;
        let mut active_model = model.into_active_model();
        active_model.status_loan = Set(status_loan);
        active_model.update(trx).await?;
        Ok(())
    }

    async fn delete(&self, trx: &DatabaseTransaction, slug: &str) -> Result<(), AppError> {
        let model = Books::delete_many()
            .filter(books::Column::Slug.eq(slug))
            .exec(trx)
            .await?;

        if model.rows_affected == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }
}
