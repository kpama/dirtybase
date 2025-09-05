use sha2::{Digest, Sha256};

pub fn hash_str(subject: &str) -> String {
    hash_bytes(subject.as_bytes())
}

pub fn hash_string(subject: String) -> String {
    hash_bytes(subject.as_bytes())
}

pub fn hash_bytes(subject: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(subject);
    format!("{:x}", hash.finalize())
}

pub fn hash_struct<S: serde::Serialize>(subject: &S) -> String {
    let s = serde_json::to_string(subject).unwrap();
    hash_bytes(s.as_bytes())
}
