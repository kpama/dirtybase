use std::sync::Arc;

use dirtybase_contract::{
    auth::{UserProviderManager, UserProviderService},
    config::DirtyConfig,
    http::WebMiddlewareManager,
    ExtensionSetup,
};

use crate::{
    middlewares::{
        handle_basic_auth_middleware, handle_jwt_auth_middleware, handle_normal_auth_middleware,
    },
    AuthConfig, AuthManager,
};

#[derive(Debug, Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, base_config: &DirtyConfig) {
        let config = AuthConfig::from(base_config);

        busybody::helpers::register_type(Arc::new(AuthManager::new(config)));
        // TODO: Move the provider to the auth manager
        busybody::helpers::register_type(Arc::new(UserProviderService::new(UserProviderManager)));
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
