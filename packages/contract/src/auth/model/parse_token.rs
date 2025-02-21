use std::fmt::{Debug, Display};

use anyhow::anyhow;
use argon2::password_hash::SaltString;
use crypto::aead::OsRng;
use dirtybase_helper::{
    hash::sha256,
    uuid::{uuid25_from_str, uuid_v7_from_str},
};

use crate::db::types::ArcUuid7;

pub struct ParseToken {
    base: String,
    hash: String,
    id: ArcUuid7,
}

impl ParseToken {
    pub fn base(&self) -> &String {
        &self.base
    }

    pub fn hash(&self) -> &String {
        &self.hash
    }

    pub fn id(&self) -> ArcUuid7 {
        self.id.clone()
    }

    pub fn is_valid(&self, user_salt: &str) -> bool {
        let salt = format!("{}{}{}", &self.base, user_salt, &self.id);
        self.hash == sha256::hash_str(&salt)
    }

    pub fn generate_token(salt: &str, auth_user_id: ArcUuid7) -> String {
        let base = SaltString::generate(&mut OsRng).to_string();
        let uuid_25 = auth_user_id.to_uuid25();
        let salt = format!("{}{}{}", &base, salt, auth_user_id);
        format!("{}.{}.{}", &base, sha256::hash_str(&salt), uuid_25)
    }
}

impl Display for ParseToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.xxxxxxxxxxx", &self.base, &self.hash)
    }
}
impl Debug for ParseToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self)
    }
}

impl TryFrom<String> for ParseToken {
    type Error = anyhow::Error;

    fn try_from(token: String) -> Result<Self, Self::Error> {
        let parts = token.split('.').map(String::from).collect::<Vec<String>>();

        if parts.len() != 3 {
            return Err(anyhow!("wrong token parts"));
        }
        let mut pieces = parts.into_iter();

        let base_piece = pieces.next();
        let hash_piece = pieces.next();
        let id_piece = pieces.next();

        if base_piece.is_none() || hash_piece.is_none() || id_piece.is_none() {
            return Err(anyhow!("one or parts are missing"));
        }

        let base = base_piece.unwrap();
        let hash = hash_piece.unwrap();
        let id = id_piece.unwrap();

        if let Some(uuid) = uuid25_from_str(&id) {
            let uuid7 = match uuid_v7_from_str(uuid.to_hyphenated().to_string().as_str()) {
                Some(id) => id,
                None => return Err(anyhow!("invalid UUID7")),
            };

            return Ok(ParseToken {
                base,
                hash,
                id: uuid7.into(),
            });
        }

        Err(anyhow!("could not parse token"))
    }
}
