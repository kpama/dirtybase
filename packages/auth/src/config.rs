use std::sync::Arc;

use busybody::async_trait;
use dirtybase_contract::{
    app::Context,
    config::{ConfigResult, DirtyConfig, TryFromDirtyConfig},
};

pub const MEMORY_STORAGE: &'static str = "memory";
pub const DATABASE_STORAGE: &'static str = "database";
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AuthConfig {
    enable: bool,
    storage: Arc<String>,
    signin_form_route: Arc<String>,
    auth_route: Arc<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enable: false,
            storage: "memory".to_string().into(),
            signin_form_route: Arc::new(String::from("auth:signin-form")),
            auth_route: Arc::new(String::from("auth::do-signin")),
        }
    }
}

#[async_trait]
impl TryFromDirtyConfig for AuthConfig {
    type Returns = Self;

    async fn from_config(base: &DirtyConfig, _ctx: &Context) -> ConfigResult<Self::Returns> {
        let mut config = base
            .optional_file("auth.toml", Some("DTY_AUTH"))
            .build()
            .await?
            .try_deserialize::<Self>()?;
        config.storage = config.storage.to_lowercase().trim().to_string().into();

        Ok(config)
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

    pub fn storage(&self) -> Arc<String> {
        self.storage.clone()
    }

    pub fn storage_ref(&self) -> &Arc<String> {
        &self.storage
    }
}
