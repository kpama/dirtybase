use super::AuthUserStorageProvider;

#[derive(Clone)]
pub struct AuthService(AuthUserStorageProvider);

impl AuthService {
    pub fn new(p: AuthUserStorageProvider) -> Self {
        Self(p)
    }
}
