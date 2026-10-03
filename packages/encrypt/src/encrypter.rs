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
    /// If the key is an empty slice, a random key will be generated and printed out
    pub fn new(key: &[u8], previous_keys: Option<Vec<Vec<u8>>>) -> Self {
        let mut generated_key = Vec::new();
        if key.len() == 0 {
            generated_key = Self::generate_aes256gcm_key();
            let key_string = Self::key_to_env_value(&generated_key);
            println!("-----------------------------------------------");
            println!("                   WARNING!                    ");
            println!("-----------------------------------------------");
            println!(
                "Encryption key is not set.\nOne was generated.\nStore it in your .env:\n\n{}",
                &key_string
            );
            println!("***********************************************");
        }

        Self {
            key: Arc::new(if key.len() > 0 {
                key.to_vec()
            } else {
                generated_key
            }),
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

    pub fn key_to_env_value(key: &[u8]) -> String {
        format!("base64:{}", base64ct::Base64::encode_string(key))
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
