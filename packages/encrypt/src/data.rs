use std::fmt::Display;

use base64ct::Encoding;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct EncryptedData {
    d: Vec<u8>, // Data
    n: Vec<u8>, // nonce
}

impl EncryptedData {
    pub fn new(content: Vec<u8>, nonce: Vec<u8>) -> Self {
        Self {
            d: content,
            n: nonce,
        }
    }

    pub fn data_ref(&self) -> &[u8] {
        &self.d
    }

    pub fn nonce_ref(&self) -> &[u8] {
        &self.n
    }

    /// Returns the inner contents
    ///
    /// (data, nonce)
    pub fn take(self) -> (Vec<u8>, Vec<u8>) {
        (self.d, self.n)
    }
}

impl Display for EncryptedData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = serde_json::to_vec(&self).unwrap();
        write!(f, "{}", base64ct::Base64::encode_string(&string))
    }
}

impl From<String> for EncryptedData {
    fn from(value: String) -> Self {
        match base64ct::Base64::decode_vec(&value) {
            Ok(d) => serde_json::from_slice(&d).unwrap_or_default(),
            Err(e) => {
                // FIXME: Translation
                tracing::error!("could not decrypt data: {}", e);
                Self::default()
            }
        }
    }
}
