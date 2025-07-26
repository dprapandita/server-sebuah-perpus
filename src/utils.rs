use rand::distr::Alphanumeric;
use rand::Rng;
use regex::Regex;

pub fn slugify(text: &str) -> String {
    let re = Regex::new(r"[^\w\s-]").unwrap();
    let cleaned = re.replace_all(text, "");
    cleaned.trim()
        .to_lowercase()
        .replace(' ', "-")
        .replace("--", "-")
}

pub fn generate_password(len: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}