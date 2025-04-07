use std::sync::Arc;

use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit};
use base64ct::Encoding;
use crypto::aead::{self, Aead, OsRng, generic_array::GenericArray};

pub struct Encrypter {
    key: Arc<Vec<u8>>,
    previous_keys: Arc<Option<Vec<Vec<u8>>>>,
}

impl Encrypter {
    pub fn new(key: &[u8], previous_keys: Option<Vec<Vec<u8>>>) -> Self {
        Self {
            key: Arc::new(key.iter().map(|e| e.clone()).collect::<Vec<u8>>()),
            previous_keys: Arc::new(previous_keys),
        }
    }

    pub fn encrypt_str(&self, data: &str) -> aead::Result<Vec<u8>> {
        self.encrypt(data.into())
    }

    pub fn encrypt(&self, data: Vec<u8>) -> aead::Result<Vec<u8>> {
        let aes256gcm = Aes256GcmEncrypter {
            key: self.key.clone(),
            previous_keys: self.previous_keys.clone(),
        };
        // AES256GCM encrypt
        aes256gcm.encrypt(data)
    }

    pub fn decrypt(&self, input: &[u8]) -> aead::Result<Vec<u8>> {
        let aes256gcm = Aes256GcmEncrypter {
            key: self.key.clone(),
            previous_keys: self.previous_keys.clone(),
        };
        aes256gcm.decrypt(input)
    }

    pub fn generate_aes256gcm_key() -> Vec<u8> {
        Aes256Gcm::generate_key(OsRng).to_vec()
    }

    pub fn generate_aes256gcm_key_string() -> String {
        base64ct::Base64::encode_string(&Self::generate_aes256gcm_key())
    }
}

struct Aes256GcmEncrypter {
    key: Arc<Vec<u8>>,
    previous_keys: Arc<Option<Vec<Vec<u8>>>>,
}

impl Aes256GcmEncrypter {
    fn encrypt(&self, data: Vec<u8>) -> aead::Result<Vec<u8>> {
        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(&key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let data = cipher.encrypt(&nonce, &*data)?;

        let mut full = nonce.to_vec();
        full.extend_from_slice(&data);
        Ok(full)
    }

    fn decrypt(&self, input: &[u8]) -> aead::Result<Vec<u8>> {
        if input.is_empty() {
            tracing::error!("could not decrypt data or nonce is empty");
            return Err(aead::Error);
        }

        let (nonce, ciphered) = input.split_at(12);
        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(&key);

        if let Ok(d) = cipher.decrypt(GenericArray::from_slice(nonce), ciphered) {
            return Ok(d);
        }

        tracing::trace!("fallback to previous keys");

        if self.previous_keys.is_none() {
            tracing::error!("decryption failed. no previous keys found");
            return Err(aead::Error);
        }

        for keys in self.previous_keys.as_ref().iter() {
            for a_key in keys {
                let key = Key::<Aes256Gcm>::from_slice(&a_key);
                let cipher = Aes256Gcm::new(&key);

                let d = cipher.decrypt(GenericArray::from_slice(nonce), ciphered);
                if d.is_ok() {
                    return d;
                }
            }
        }

        tracing::error!("decryption failed. used all possible keys");
        Err(aead::Error)
    }
}
