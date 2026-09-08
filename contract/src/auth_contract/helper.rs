use rand::RngExt;
use rand::distr::Alphanumeric;

use crate::db_contract::types::ArcUuid7;

use super::ParseToken;

pub fn generate_user_token(salt: &str, auth_user_id: &ArcUuid7) -> String {
    ParseToken::generate_token(salt, auth_user_id)
}

pub fn parse_user_token(token: &str) -> Result<ParseToken, anyhow::Error> {
    ParseToken::try_from(token.to_string())
}

pub fn generate_salt() -> String {
    let mut rng = rand::rng();
    (0..=15).map(|_| rng.sample(Alphanumeric) as char).collect()
}
