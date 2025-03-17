use argon2::password_hash::SaltString;
use crypto::aead::OsRng;

use crate::db::types::ArcUuid7;

use super::ParseToken;

pub fn generate_user_token(salt: &str, auth_user_id: &ArcUuid7) -> String {
    ParseToken::generate_token(salt, auth_user_id)
}

pub fn parse_user_token(token: &str) -> Result<ParseToken, anyhow::Error> {
    ParseToken::try_from(token.to_string())
}

pub fn generate_salt() -> String {
    SaltString::generate(&mut OsRng).to_string()
}
