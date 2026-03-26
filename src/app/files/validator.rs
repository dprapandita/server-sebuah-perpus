use crate::utils::slugify;
use std::path::{Component, Path};

/// Max ukuran per chunk: 2 MB
pub const MAX_CHUNK_SIZE: u64 = 2 * 1024 * 1024;
/// Max ukuran total upload: 50 MB
pub const MAX_UPLOAD_SIZE: u64 = 50 * 1024 * 1024;

/// Ekstensi file yang diizinkan
const ALLOWED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "mp4", "pdf", "epub"];

/// MIME type yang diizinkan berdasarkan magic bytes
const ALLOWED_MIME_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/gif",
    "application/pdf",
    "application/epub+zip",
    "video/mp4",
];

/// Cek apakah semua ekstensi dalam `exts` ada di daftar yang diizinkan
pub fn allowed_extensions(exts: &[&str]) -> bool {
    exts.iter().all(|ext| ALLOWED_EXTENSIONS.contains(ext))
}

/// Validasi MIME type file dari magic bytes-nya
pub fn validate_book_mime(data: &[u8]) -> Result<(), String> {
    let kind = infer::get(data)
        .ok_or_else(|| "Cannot detect file type (unrecognized magic bytes)".to_string())?;

    let mime = kind.mime_type();
    if ALLOWED_MIME_TYPES.contains(&mime) {
        Ok(())
    } else {
        Err(format!("File type not allowed: {}", mime))
    }
}

/// Validasi ukuran chunk tidak melebihi `max_size_mb` MB
pub fn validate_chunk_size(data: &[u8], max_size_mb: u64) -> Result<(), String> {
    let max_bytes = (max_size_mb * 1024 * 1024) as usize;
    if data.len() > max_bytes {
        Err(format!("Chunk too large (max {} MB)", max_size_mb))
    } else {
        Ok(())
    }
}

/// Sanitasi nama file: slugify stem + pertahankan ekstensi lowercase
pub fn sanitize_filename(file_name: &str) -> String {
    let path = Path::new(file_name);

    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");

    let slug = slugify(stem);

    if ext.is_empty() {
        slug
    } else {
        format!("{}.{}", slug, ext)
    }
}

/// Buat nama file dengan prefix timestamp
pub fn timestamped_filename(file_name: &str) -> String {
    let ts = chrono::Utc::now().format("%Y%m%d%H%M%S");
    format!("{}_{}", ts, sanitize_filename(file_name))
}

/// Pastikan path tidak mengandung traversal (`..`, `.`, `/root`)
pub fn path_is_valid(raw: &str) -> bool {
    !Path::new(raw).components().any(|c| {
        matches!(
            c,
            Component::ParentDir | Component::CurDir | Component::RootDir
        )
    })
}
