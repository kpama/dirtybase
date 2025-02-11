use crate::AuthConfig;

#[derive(Clone)]
pub struct AuthManager {
    config: AuthConfig,
}

impl AuthManager {
    pub fn new(config: AuthConfig) -> Self {
        Self { config }
    }
    pub fn is_enable(&self) -> bool {
        self.config.enable()
    }

    pub async fn validate_jwt(&self, jwt: String) -> bool {
        false
    }
}
