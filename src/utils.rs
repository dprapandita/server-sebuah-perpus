use std::sync::OnceLock;
use chrono::NaiveDateTime;
use rand::distr::Alphanumeric;
use rand::Rng;
use regex::Regex;
use crate::core::error::AppError;

pub fn slugify(text: &str) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"[^\w\s-]").unwrap());
    let cleaned = re.replace_all(text, "");
    cleaned.trim()
        .to_lowercase()
        .replace(' ', "-")
        .replace("--", "-")
}

pub async fn generate_unique_slug(title: &str) -> String
{
    let base_slug = slugify(title);
    let suffix = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect::<String>();
    format!("{}-{}", base_slug, suffix.to_lowercase())
}

pub fn generate_password(len: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

pub fn generate_username() -> Result<String, AppError> {
    let suffix = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect::<String>();
    Ok(suffix)
}

pub fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

pub fn normalize_phone(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

pub fn local_time() -> NaiveDateTime {
    chrono::Local::now().naive_local()
}

/// Membersihkan dan memvalidasi apakah string adalah ISBN yang sah
pub fn sanitize_isbn(raw_id: Option<String>) -> Option<String> {
    // Jika datanya kosong dari awal, langsung return None
    let id = raw_id?;

    // Hapus tanda strip (-) atau spasi yang biasa ada di penulisan ISBN
    let clean_id = id.replace("-", "").replace(" ", "");

    let is_valid_length = clean_id.len() == 10 || clean_id.len() == 13;
    let is_alphanumeric = clean_id.chars().all(|c| c.is_ascii_digit() || c == 'X' || c == 'x');

    if is_valid_length && is_alphanumeric {
        Some(clean_id) // Lolos validasi, kembalikan ISBN bersih
    } else {
        None // Ini pasti UUID atau data sampah, jadikan NULL di database
    }
}