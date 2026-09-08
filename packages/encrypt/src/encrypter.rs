use std::sync::Arc;

use aes_gcm::{
    Aes256Gcm, Key, KeyInit, Nonce,
    aead::{Aead, Generate},
};
use anyhow::anyhow;
use base64ct::Encoding;

pub struct Encrypter {
    key: Arc<Vec<u8>>,
    previous_keys: Arc<Option<Vec<Vec<u8>>>>,
}

impl Encrypter {
    pub fn new(key: &[u8], previous_keys: Option<Vec<Vec<u8>>>) -> Self {
        if key.len() == 0 {
            panic!("encryption key is not set. Generate a valid key");
        }

        Self {
            key: Arc::new(key.to_vec()),
            previous_keys: Arc::new(previous_keys),
        }
    }

    pub fn encrypt_str(&self, data: &str) -> anyhow::Result<Vec<u8>> {
        self.encrypt(data.into())
    }

    pub fn encrypt(&self, data: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        let aes256gcm = Aes256GcmEncrypter {
            key: self.key.clone(),
            previous_keys: self.previous_keys.clone(),
        };
        aes256gcm.encrypt(data).map_err(|e| anyhow!(e))
    }

    pub fn decrypt(&self, input: &[u8]) -> anyhow::Result<Vec<u8>> {
        let aes256gcm = Aes256GcmEncrypter {
            key: self.key.clone(),
            previous_keys: self.previous_keys.clone(),
        };
        aes256gcm.decrypt(input)
    }

    pub fn generate_aes256gcm_key() -> Vec<u8> {
        Key::<Aes256Gcm>::generate().to_vec()
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
    fn key_into_aes_key(&self) -> Key<Aes256Gcm> {
        self.key
            .clone()
            .as_array()
            .cloned()
            .unwrap_or_default()
            .try_into()
            .expect("could not generate encryption key from current value")
    }
    fn encrypt(&self, data: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        let key = self.key_into_aes_key();
        let cipher = Aes256Gcm::new(&key);
        let nonce = Nonce::generate();
        let data = cipher.encrypt(&nonce, &*data).map_err(|e| anyhow!(e))?;

        let mut full = nonce.to_vec();
        full.extend_from_slice(&data);
        Ok(full)
    }

    fn decrypt(&self, input: &[u8]) -> anyhow::Result<Vec<u8>> {
        if input.is_empty() {
            return Err(anyhow!("could not descrypt an empty slice"));
        }

        let (nonce, ciphered) = input.split_at(12);
        let key = self.key_into_aes_key();
        let cipher = Aes256Gcm::new(&key);
        let n = Nonce::try_from(nonce).map_err(|e| anyhow!(e))?;

        if let Ok(d) = cipher.decrypt(&n, ciphered) {
            return Ok(d);
        }

        tracing::trace!("fallback to previous keys");

        if self.previous_keys.is_none() {
            return Err(anyhow!("decryption failed. no previous keys found"));
        }

        for keys in self.previous_keys.as_ref().iter() {
            for a_key in keys {
                let key: Key<Aes256Gcm> = a_key
                    .clone()
                    .as_array()
                    .cloned()
                    .unwrap_or_default()
                    .try_into()
                    .map_err(|_| anyhow!("could not generate encryption key from current value"))?;
                let cipher = Aes256Gcm::new(&key);
                let n = Nonce::try_from(nonce)
                    .map_err(|_| anyhow!("could not create nonce from slice"))?;

                let d = cipher.decrypt(&n, ciphered);
                if let Ok(d) = d {
                    return Ok(d);
                }
            }
        }

        Err(anyhow!("decryption failed. used all possible keys"))
    }
}
