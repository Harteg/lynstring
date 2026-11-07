use sha2::{Sha256, Digest};

/// Normalize text for comparison (trim whitespace)
/// Canonicalize text for stable hashing
/// - Trim whitespace
/// - Replace multiple whitespace with single space
/// - Normalize line breaks
pub fn canonicalize_text(text: &str) -> String {
    text.trim()
        .lines()
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Compute stable hash for a string: SHA256(key + "|" + canonicalized_text)
pub fn compute_string_hash(key: &str, text: &str) -> String {
    let canonical = canonicalize_text(text);
    let input = format!("{}|{}", key, canonical);
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generate a hash of the project path for use as filename
pub fn project_path_hash(project_path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(project_path.as_bytes());
    format!("{:x}", hasher.finalize())
}

