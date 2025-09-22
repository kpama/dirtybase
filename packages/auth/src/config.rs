use std::sync::Arc;

use busybody::async_trait;
use dirtybase_contract::{
    app_contract::Context,
    config_contract::{ConfigResult, DirtyConfig, TryFromDirtyConfig},
};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AuthConfig {
    enable: bool,
    storage: Arc<String>,
    allow_self_signup: bool,
    signin_form_route: Arc<String>,
    auth_route: Arc<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enable: false,
            storage: "memory".to_string().into(),
            allow_self_signup: false,
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

    pub fn allow_self_signup(&self) -> bool {
        self.allow_self_signup
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
    pub fn storage_as_str(&self) -> &str {
        self.storage_ref().as_str()
    }
}
