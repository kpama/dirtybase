use dirtybase_contract::{http::MiddlewareManager, ExtensionSetup};

pub struct Extension;

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    fn register_web_middlewares(&self, mut _manager: MiddlewareManager) -> MiddlewareManager {
        _manager
    }
}
