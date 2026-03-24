use crate::core::error::AppError;
use epub::doc::EpubDoc;
use serde::{Deserialize, Serialize};

/// Metadata hasil parsing dari file EPUB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubMetadata {
    pub title: String,
    pub author: String,
    pub publisher: String,
    pub isbn: String,
    pub description: String,
    pub cover_image: Option<Vec<u8>>,
    pub cover_mime: Option<String>,
}

/// Parse file EPUB dan extract metadata (title, author, publisher, ISBN, description, cover).
///
/// Menggunakan crate `epub` untuk membuka file dan `ammonia` untuk sanitize HTML
/// dari field description.
pub fn parse_epub(file_path: &str) -> Result<EpubMetadata, AppError> {
    let mut doc = EpubDoc::new(file_path)
        .map_err(|e| AppError::InternalError(format!("Gagal membuka EPUB: {:?}", e)))?;

    let title = doc
        .get_title()
        .ok_or_else(|| AppError::Validation("EPUB tidak memiliki title".to_string()))?;

    let author = doc
        .mdata("creator")
        .map(|m| m.value.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let publisher = doc
        .mdata("publisher")
        .map(|m| m.value.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let isbn = doc
        .mdata("identifier")
        .map(|m| m.value.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let raw_description = doc
        .mdata("description")
        .map(|m| m.value.clone())
        .unwrap_or_default();

    // Sanitize HTML dari description pakai ammonia
    let description = if raw_description.is_empty() {
        String::new()
    } else {
        ammonia::clean(&raw_description)
    };

    // Extract cover image
    let (cover_image, cover_mime) = match doc.get_cover() {
        Some((data, mime)) => (Some(data), Some(mime)),
        None => (None, None),
    };

    Ok(EpubMetadata {
        title,
        author,
        publisher,
        isbn,
        description,
        cover_image,
        cover_mime,
    })
}
