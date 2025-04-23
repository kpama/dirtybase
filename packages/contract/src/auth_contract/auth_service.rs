use std::sync::OnceLock;

use super::{AuthUser, AuthUserStorageProvider};

#[derive(Clone)]
pub struct AuthService {
    _storage: AuthUserStorageProvider,
    _user: OnceLock<AuthUser>,
}

impl AuthService {
    pub fn new(storage: AuthUserStorageProvider) -> Self {
        Self {
            _storage: storage,
            _user: OnceLock::default(),
        }
    }
}
