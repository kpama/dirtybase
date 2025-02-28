use super::AuthUserStorageProvider;

pub struct AuthService(AuthUserStorageProvider);

impl AuthService {
    pub fn new(p: AuthUserStorageProvider) -> Self {
        Self(p)
    }
}
