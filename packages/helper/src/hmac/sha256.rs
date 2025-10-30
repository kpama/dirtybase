use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::base64;

pub fn hash_bytes(key: &[u8], subject: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let mut mac = match Hmac::<Sha256>::new_from_slice(key) {
        Ok(mac) => mac,
        Err(e) => return Err(anyhow::anyhow!(e)),
    };

    mac.update(subject);
    let result = mac.finalize();
    let hash = result.into_bytes().to_vec();

    Ok(hash)
}

pub fn hash_str(key: &[u8], subject: &str) -> Result<Vec<u8>, anyhow::Error> {
    hash_bytes(key, subject.as_bytes())
}

pub fn hash_str_to_hex(key: &[u8], subject: &str) -> Result<String, anyhow::Error> {
    hash_str(key, subject).map(hex::encode)
}

pub fn hash_str_to_base64(key: &[u8], subject: &str) -> Result<String, anyhow::Error> {
    hash_str(key, subject).map(|bytes| base64::encode(hex::encode(bytes).as_bytes()))
}
