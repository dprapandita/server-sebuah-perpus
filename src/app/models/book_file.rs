use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Representasi file buku dari database.
#[derive(Debug, Serialize, Deserialize)]
pub struct BookFile {
    pub id: Uuid,
    pub book_id: Uuid,
    pub file_path: String,
    pub file_format: String,
    pub file_size: i64,
    pub file_checksum: String,
}

/// Payload untuk membuat book_file baru.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBookFilePayload {
    pub book_id: Uuid,
    pub file_path: String,
    pub file_format: String,
    pub file_size: i64,
    pub file_checksum: String,
}

/// Aggregate yang menggabungkan satu buku dengan semua file-nya (relasi 1:N).
#[derive(Debug, Serialize, Deserialize)]
pub struct BookAggregate {
    pub book: entity::books::Model,
    pub book_files: Vec<entity::book_files::Model>,
}

/// Detail lengkap sebuah book_file beserta info buku terkait,
/// dipakai untuk menampilkan seluruh isi book_file secara flat.
#[derive(Debug, Serialize, Deserialize)]
pub struct BookFileDetail {
    pub id: Uuid,
    pub book_id: Uuid,
    pub book_title: String,
    pub cover_image: Option<String>,
    pub book_slug: String,
    pub file_path: String,
    pub file_format: String,
    pub file_size: i64,
    pub file_checksum: String,
}

impl From<entity::book_files::Model> for BookFile {
    fn from(model: entity::book_files::Model) -> Self {
        Self {
            id: model.id,
            book_id: model.book_id,
            file_path: model.file_path,
            file_format: model.file_format,
            file_size: model.file_size,
            file_checksum: model.checksum,
        }
    }
}
