mod migration;
use std::sync::Arc;

use dirtybase_contract::{
    ExtensionMigrations, ExtensionSetup, config::DirtyConfig, http::WebMiddlewareManager,
};

use crate::{
    AuthConfig, AuthManager,
    middlewares::{
        handle_basic_auth_middleware, handle_jwt_auth_middleware, handle_normal_auth_middleware,
    },
};

#[derive(Debug, Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, base_config: &DirtyConfig) {
        let config = AuthConfig::from_dirty_config(base_config).await;

        busybody::helpers::register_type(Arc::new(AuthManager::new(config))).await;
    }

    fn migrations(&self) -> Option<ExtensionMigrations> {
        migration::setup()
    }

    fn register_web_middlewares(&self, mut manager: WebMiddlewareManager) -> WebMiddlewareManager {
        manager
            .register("auth::basic", |router| {
                router.middleware(handle_basic_auth_middleware)
            })
            .register("auth::jwt", |router| {
                router.middleware(handle_jwt_auth_middleware)
            })
            .register("auth::normal", |router| {
                println!("registering the normal auth middleware");
                router.middleware(handle_normal_auth_middleware)
            });

        manager
    }
}
