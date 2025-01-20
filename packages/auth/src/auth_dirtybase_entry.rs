use dirtybase_contract::{
    auth::{UserProviderManager, UserProviderService},
    dirtybase_config::DirtyConfig,
    http::WebMiddlewareManager,
    ExtensionSetup,
};

use crate::middlewares::{BasicAuthMiddleware, JWTAuthMiddleware, NormalAuthMiddleware};

pub struct Extension;

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&self, _config: &DirtyConfig) {
        busybody::helpers::register_service(UserProviderService::new(UserProviderManager));
    }

    fn register_web_middlewares(&self, mut manager: WebMiddlewareManager) -> WebMiddlewareManager {
        manager
            .register("auth:basic", BasicAuthMiddleware)
            .register("auth:jwt", JWTAuthMiddleware)
            .register("auth:normal", NormalAuthMiddleware);

        manager
    }
}
