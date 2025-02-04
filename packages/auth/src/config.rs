use std::sync::Arc;

use dirtybase_contract::config::DirtyConfig;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AuthConfig {
    enable: bool,
    signin_form_route: Arc<String>,
    auth_route: Arc<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enable: false,
            signin_form_route: Arc::new(String::from("auth:signin-form")),
            auth_route: Arc::new(String::from("auth::do-signin")),
        }
    }
}

impl AuthConfig {
    pub fn enable(&self) -> bool {
        self.enable
    }

    pub fn signin_form_route(&self) -> Arc<String> {
        self.signin_form_route.clone()
    }

    pub fn auth_route(&self) -> Arc<String> {
        self.auth_route.clone()
    }
}

impl From<&DirtyConfig> for AuthConfig {
    fn from(base: &DirtyConfig) -> Self {
        base.optional_file("auth.toml", Some("DTY_AUTH"))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }
}
