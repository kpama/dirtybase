use std::sync::Arc;

use dirtybase_contract::config::DirtyConfig;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AuthUserStorageDriver {
    #[serde(alias = "custom")]
    Custom(String),
    #[serde(alias = "database")]
    Database,
    #[serde(alias = "memory")]
    Memory,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AuthConfig {
    enable: bool,
    storage: AuthUserStorageDriver,
    signin_form_route: Arc<String>,
    auth_route: Arc<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enable: false,
            storage: AuthUserStorageDriver::Memory,
            signin_form_route: Arc::new(String::from("auth:signin-form")),
            auth_route: Arc::new(String::from("auth::do-signin")),
        }
    }
}

impl AuthConfig {
    pub fn is_enabled(&self) -> bool {
        self.enable
    }

    pub fn signin_form_route(&self) -> Arc<String> {
        self.signin_form_route.clone()
    }

    pub fn auth_route(&self) -> Arc<String> {
        self.auth_route.clone()
    }

    pub fn storage(&self) -> AuthUserStorageDriver {
        self.storage.clone()
    }

    pub fn storage_ref(&self) -> &AuthUserStorageDriver {
        &self.storage
    }

    pub async fn from_dirty_config(base: &DirtyConfig) -> Self {
        base.optional_file("auth.toml", Some("DTY_AUTH"))
            .build()
            .await
            .unwrap()
            .try_deserialize()
            .unwrap()
    }
}
