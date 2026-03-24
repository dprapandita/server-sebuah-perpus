use crate::app::files::validator::{path_is_valid, sanitize_filename};
use crate::app::hashing::hash::hash_file;
use axum::http::{header, StatusCode};
use sha3::{Digest, Sha3_256};
use std::path::PathBuf;
use axum::body::Body;
use axum::response::Response;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tracing::info;
use crate::core::error::AppError;
use tokio_util::io::ReaderStream;

type FileResult<T> = Result<T, (StatusCode, String)>;

/// Pointing ke folder `storage/` relatif dari current dir
pub fn path_storage(sub_path: &str) -> Option<PathBuf> {
    if sub_path.contains("..") {
        return None;
    }
    let base_path = std::env::current_dir().expect("Failed to get current directory");
    let full_path = base_path.join("storage").join(sub_path);

    if full_path.starts_with(base_path.join("storage")) {
        Some(full_path)
    } else {
        None
    }
}

/// Buat direktori parent dari `path` jika belum ada
async fn ensure_parent_dir(path: &PathBuf) -> FileResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create output dir: {}", e),
            )
        })?;
    }
    Ok(())
}

/// Tulis bytes ke path, lalu simpan hash ke `<path>.hash`
/// Return: hash string
async fn write_bytes_and_hash(output_path: &PathBuf, data: &[u8]) -> FileResult<String> {
    ensure_parent_dir(output_path).await?;

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output_path)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", e)))?;

    file.write_all(data).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to write file: {}", e),
        )
    })?;

    info!("{:?}", output_path);
    info!("Entering hashes mode");

    let hash = hash_file(data).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to hash file: {}", e),
        )
    })?;

    let hash_path = output_path.with_extension("hash");
    fs::write(&hash_path, hash.as_bytes()).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to write hash file: {}", e),
        )
    })?;

    Ok(hash)
}

/// Bangun nama file final: `<timestamp>_<sanitized_name>`
fn make_final_filename(file_name: &str) -> String {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let sanitized = sanitize_filename(file_name);
    format!("{}_{}", timestamp, sanitized)
}

/// Rakit chunks dari `path/<i>` menjadi satu Vec<u8>
async fn collect_chunks(path: &str, total_chunks: usize) -> FileResult<Vec<u8>> {
    let mut buf = Vec::new();
    for i in 0..total_chunks {
        let chunk_path = match path_storage(&format!("{}/{}", path, i)) {
            Some(path) => path,
            None => return Err((StatusCode::BAD_REQUEST, format!("Invalid chunk path at index {}", i))),
        };
        let chunk = fs::read(&chunk_path).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read chunk {}: {}", i, e),
            )
        })?;
        buf.extend_from_slice(&chunk);
    }
    Ok(buf)
}

/// Gabungkan chunks menjadi satu file, hash, lalu hapus dir chunk.
/// Return: `(relative_path, hash)`
pub async fn write_file(
    path: &str,
    file_name: &str,
    total_chunks: usize,
    output_out: Option<&str>,
) -> FileResult<(String, String)> {
    if !path_is_valid(path) {
        info!("{:?}", path);
        return Err((StatusCode::NO_CONTENT, "Invalid path".to_owned()));
    }

    let final_file_name = make_final_filename(file_name);
    let relative_path = if let Some(subdir) = output_out {
        format!("uploads/{}/{}", subdir, final_file_name)
    } else {
        format!("uploads/{}", final_file_name)
    };

    let output_path = match path_storage(&relative_path) {
        Some(path) => path,
        None => return Err((StatusCode::INTERNAL_SERVER_ERROR, relative_path)),
    };
    let data = collect_chunks(path, total_chunks).await?;
    let hash = write_bytes_and_hash(&output_path, &data).await?;
    let temp_dir_path = match path_storage(path) {
        Some(p) => p,
        None => return Err((StatusCode::INTERNAL_SERVER_ERROR, "Invalid temp directory path".to_owned())),
    };

    fs::remove_dir_all(temp_dir_path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to remove directory: {}", e),
        )
    })?;

    Ok((relative_path, hash))
}

/// Tulis file langsung dari bytes (non-chunked).
/// Return: `(relative_path, hash, file_size)`
pub async fn write_file_direct(
    file_name: &str,
    file_bytes: &[u8],
    output_dir: &str,
) -> FileResult<(String, String, i64)> {
    let final_file_name = make_final_filename(file_name);
    let relative_path = format!("uploads/{}/{}", output_dir, final_file_name);
    let output_path = match path_storage(&relative_path) {
        Some(path) => path,
        None => return Err((StatusCode::BAD_REQUEST, "Invalid storage path generated".to_owned())),
    };

    let hash = write_bytes_and_hash(&output_path, file_bytes).await?;
    let file_size = file_bytes.len() as i64;

    Ok((relative_path, hash, file_size))
}

/// Generate unique 8-byte hex ID dari kombinasi title + publisher + filename
pub fn generate_chunk_dir_id(title: &str, publisher: &str, filename: &str) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(title);
    hasher.update(publisher);
    hasher.update(filename);
    hex::encode(&hasher.finalize()[..8])
}

pub async fn build_stream_and_response(
    full_path: PathBuf,
    custom_filename: Option<String>,
) -> Result<Response, AppError> {
    // 1. Pastikan file benar-benar ada sebelum dibuka
    if !full_path.exists() || !full_path.is_file() {
        return Err(AppError::FileNotFound);
    }

    // 2. Buka file secara asynchronous
    let file = fs::File::open(&full_path).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Gagal membuka file: {}", e))
    }).expect("Failed to open file");

    // 3. Ubah file menjadi stream body
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // 4. Deteksi Content-Type dari ekstensi file
    let ext = full_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let content_type = match ext {
        "epub" => "application/epub+zip",
        "pdf"  => "application/pdf",
        "jpg" | "jpeg" => "image/jpeg",
        "png"  => "image/png",
        _      => "application/octet-stream",
    };

    // 5. Tentukan nama file yang akan muncul saat didownload user
    let filename = custom_filename.unwrap_or_else(|| {
        full_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("document")
            .to_string()
    });

    // 6. Bangun Response Axum
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(body)
        .unwrap(); // Unwrap aman di sini karena header statis

    Ok(response)
}

pub async fn delete_file(relative_path: &str) -> FileResult<()> {
    let full_path = match path_storage(relative_path) {
        Some(path) => path,
        None => return Err((StatusCode::BAD_REQUEST, "Invalid file path".to_owned())),
    };

    if !full_path.exists() || !full_path.is_file() {
        return Err((StatusCode::NOT_FOUND, "File physically missing".to_owned()));
    }

    fs::remove_file(&full_path).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete file: {}", e))
    })?;

    Ok(())
}

pub async fn read_file_stream(
    relative_path: &str,
) -> FileResult<(ReaderStream<fs::File>, u64, String)> {

    // 1. Validasi keamanan path
    let full_path = match path_storage(relative_path) {
        Some(path) => path,
        None => return Err((StatusCode::BAD_REQUEST, "Invalid file path".to_owned())),
    };

    // 2. Cek eksistensi
    if !full_path.exists() || !full_path.is_file() {
        return Err((StatusCode::NOT_FOUND, "File physically missing".to_owned()));
    }

    // 3. Buka file
    let file = fs::File::open(&full_path).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to open file: {}", e))
    })?;

    // 4. Dapatkan ukuran file (metadata)
    let metadata = file.metadata().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read metadata: {}", e))
    })?;
    let file_size = metadata.len();

    // 5. Tentukan tipe file
    let ext = full_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let content_type = match ext {
        "epub" => "application/epub+zip",
        "pdf"  => "application/pdf",
        "jpg" | "jpeg" => "image/jpeg",
        "png"  => "image/png",
        _      => "application/octet-stream",
    }.to_string();

    // 6. Buat stream pembaca
    let stream = ReaderStream::new(file);

    // Return Stream, Size, dan Content-Type murni
    Ok((stream, file_size, content_type))
}