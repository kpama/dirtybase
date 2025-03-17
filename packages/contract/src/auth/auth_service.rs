use std::sync::OnceLock;

use super::{AuthUser, AuthUserStorageProvider};

#[derive(Clone)]
pub struct AuthService {
    storage: AuthUserStorageProvider,
    user: OnceLock<AuthUser>,
}

impl AuthService {
    pub fn new(storage: AuthUserStorageProvider) -> Self {
        Self {
            storage,
            user: OnceLock::default(),
        }
    }
}
