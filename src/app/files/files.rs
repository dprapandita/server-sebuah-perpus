use axum::body::Body;
use axum::extract::Query;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use sha3::{Digest, Sha3_256};
use slug::slugify;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tracing::info;
use crate::app::files::validator::{path_is_valid, sanitize_filename};
use crate::app::hashing::hash::hash_file;
use crate::routes::{internal_error, not_found_error};

#[derive(Deserialize)]
struct DownloadParams {
    file_name: String,
    offset: u64,
    total_chunks: usize,
}

// Function that downloads a file using a chunk mechanism
// pub fn download_file(Query(params): Query<DownloadParams>) -> impl IntoResponse {
//     let file_path = path_storage(&params.file_name);
//     let mut file = match File::open(file_path) {
//         Ok(file) => file,
//         Err(_) => return not_found_error("File not found"),
//     };
//
//     let total_size = match file.metadata() {
//         Ok(metadata) => metadata.len(),
//         Err(_) => return not_found_error("File metadata not found"),
//     };
//
//     if params.offset >= total_size {
//         return not_found_error("Offset out of bounds");
//     }
//
//     if file.seek(SeekFrom::Start(params.offset)).is_err() {
//         return not_found_error("File seek out of bounds");
//     }
//
//     let mut buffer = vec![0; params.total_chunks];
//     let bytes_read = match file.read(&mut buffer) {
//         Ok(bytes) => bytes,
//         Err(_) => return internal_error("Error reading file"),
//     };
//
//     if bytes_read == 0 {
//         return not_found_error("Empty file");
//     }
//
//     let mut headers = HeaderMap::new();
//     headers.insert("Content-Type", HeaderValue::from_static("application/octet-stream"));
//     headers.insert("Content-Disposition", HeaderValue::from_str(&format!(
//         "attachment; filename=\"{}\"", params.file_name
//     )).unwrap());
//
//     headers.insert("X-Chunk-Offset", HeaderValue::from_str(&params.offset.to_string()).unwrap());
//     headers.insert("X-Chunk-Size", HeaderValue::from_str(&params.total_chunks.to_string()).unwrap());
//     headers.insert("X-Total-Size", HeaderValue::from_str(&total_size.to_string()).unwrap());
//
//     Response::builder()
//         .status(StatusCode::PARTIAL_CONTENT)
//         .header("Content-Type", "application/octet-stream")
//         .header(
//             "Content-Disposition",
//             format!("attachment; filename=\"{}\"", params.file_name),
//         )
//         .header("X-Chunk-Offset", params.offset.to_string())
//         .header("X-Chunk-Size", bytes_read.to_string())
//         .body(Body::from(buffer[..bytes_read].to_vec()))
//         .unwrap_or_else(|_| {
//             Response::builder()
//                 .status(StatusCode::INTERNAL_SERVER_ERROR)
//                 .body(Body::from("Failed to build response"))
//                 .unwrap()
//         })
// }

/// Function that pointing to storage folder
pub fn path_storage(sub_path: &str) -> PathBuf {
    let base_path = std::env::current_dir().expect("Failed to get current directory");
    base_path.join("storage").join(sub_path)
}

/// Write a file as chunks
pub async fn write_file(path: &str, file_name: &str, total_chunks: usize, output_out: Option<&str>)
    -> Result<(String, String), (StatusCode, String)>
{
    if !path_is_valid(path) {
        info!("{:?}", path);
        return Err((StatusCode::NO_CONTENT, "Invalid path".to_owned()));
    }

    let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let sanitized_name = sanitize_filename(&file_name);
    let final_file_name = format!("{}_{}", &timestamp, sanitized_name);

    // let output_dir = Path::new(path).parent()
    //     .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Invalid chunk directory structure".to_string()))?; // keluar dari folder chunk
    // let output_file_name = output_dir.join(file_name).as_path().to_str().unwrap().to_string();
    let relative_path = match output_out {
        Some(subdir) => format!("uploads/{}/{}", subdir, final_file_name),
        None => format!("uploads/{}", final_file_name)
    };

    let output_path = path_storage(&relative_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).await.map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create output dir: {}", e))
        })?;
    }


    let mut output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&output_path).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", e)))?;

    for chunk_number in 0..total_chunks {
        // let ck_path = format!("{}/chunk/{}", path, chunk_number);
        let ck_path = format!("{}/{}", path, chunk_number);
        let chunk_path = path_storage(&ck_path);
        let chunk_data = fs::read(&chunk_path)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read chunk {}: {}", chunk_number, e)))?;
        output_file.write_all(&chunk_data)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to write chunk {}: {}", chunk_number, e)))?;
    }

    info!("{:?}", output_path);
    info!("Entering hashes mode");
    let file_bytes = fs::read(&output_path).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read file: {}", e)))?;

    let hash = hash_file(&file_bytes)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to hash file: {}", e)))?;

    info!("Entering hashes directory");
    let hash_file_path = output_path.with_extension("hash");

    fs::write(&hash_file_path, hash.as_bytes()).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to write hash file: {}", e)))?;

    fs::remove_dir_all(path_storage(path))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to remove directory: {}", e)))?;
    Ok((relative_path, hash))

}

/// Generate unique hash for directory
pub fn generate_chunk_dir_id(title: &str, publisher: &str, filename: &str) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(&title);
    hasher.update(&publisher);
    hasher.update(&filename);
    let hash = hasher.finalize();
    hex::encode(&hash[..8])
}
