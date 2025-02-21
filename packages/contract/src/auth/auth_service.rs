use anyhow::anyhow;
use validator::Validate;

use crate::db::types::ArcUuid7;

use super::{generate_salt, AuthUser, AuthUserPayload, AuthUserStorageProvider};

pub struct AuthService(AuthUserStorageProvider);

impl AuthService {
    pub fn new(p: AuthUserStorageProvider) -> Self {
        Self(p)
    }
}
