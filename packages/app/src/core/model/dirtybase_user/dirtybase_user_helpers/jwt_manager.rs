use std::collections::HashMap;

use hmac::Hmac;
use jwt::SignWithKey;
use jwt::VerifyWithKey;
use sha2::Sha256;

pub struct JWTManager {
    hmac_key: Hmac<Sha256>,
}

impl JWTManager {
    pub fn sign_to_jwt(&self, claims: HashMap<String, String>) -> Option<String> {
        match claims.sign_with_key(&self.hmac_key) {
            Ok(s) => Some(s),
            Err(e) => {
                log::error!("could not generate jwt: {}", e.to_string());
                None
            }
        }
    }

    pub fn verify_jwt(&self, jwt: &str) -> Option<HashMap<String, String>> {
        let result: Result<HashMap<String, String>, _> = jwt.verify_with_key(&self.hmac_key);
        match result {
            Ok(claim) => Some(claim),
            Err(e) => {
                log::info!("Could not verify JWT: {}", e.to_string());
                None
            }
        }
    }
}
