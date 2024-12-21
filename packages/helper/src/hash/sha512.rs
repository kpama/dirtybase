use sha2::{Digest, Sha512};

pub fn hash_str(subject: &str) -> String {
    hash_bytes(subject.as_bytes())
}

pub fn hash_bytes(subject: &[u8]) -> String {
    let mut hash = Sha512::new();
    hash.update(subject);
    format!("{:x}", hash.finalize())
}

pub fn hash_struct<S: serde::Serialize>(subject: &S) -> String {
    let s = serde_json::to_string(subject).unwrap();
    hash_bytes(s.as_bytes())
}
