use crc32fast::Hasher;
use rocket::serde::Serialize;

#[derive(Serialize)]
pub struct Audiobook {
    pub title: String,
    pub author: String,
    pub path: String,
}

#[derive(Serialize)]
pub struct AudiobookFmt {
    pub hash: String,
    pub title: String,
    pub author: String,
}

/// Computes the hash of a audiobook
pub fn compute_hash(title: String, author: String) -> String {
    let input = format!("{}{}", title, author);
    let mut hasher = Hasher::new();
    hasher.update(input.as_bytes());
    let crc = hasher.finalize();
    let hash = format!("{:x}", crc);
    hash
}
