use crate::app::models::book_file::BookFile;
use entity::generated::sea_orm_active_enums::BookStatusType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Book {
    pub id: Uuid,
    pub isbn: Option<String>,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub author: String,
    pub publisher: String,
    pub status: Option<BookStatusType>,
    pub status_loan: Option<entity::generated::sea_orm_active_enums::BookStatusLoanType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBookPayload {
    pub isbn: Option<String>,
    pub title: String,
    pub description: String,
    pub slug: String,
    pub author: String,
    pub publisher: String,
    pub cover_image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBookPayload {
    pub title: String,
    pub description: String,
    pub author: String,
    pub publisher: String,
}

/// Response yang menggabungkan Book dan BookFile untuk endpoint upload.
#[derive(Debug, Serialize, Deserialize)]
pub struct BookWithFileResponse {
    pub book: Book,
    pub book_file: BookFile,
}

impl From<entity::books::Model> for Book {
    fn from(model: entity::books::Model) -> Self {
        Self {
            id: model.id,
            title: model.title,
            isbn: model.isbn,
            slug: model.slug,
            description: model.description,
            publisher: model.publisher,
            author: model.author,
            status: model.status,
            status_loan: model.status_loan,
        }
    }
}
