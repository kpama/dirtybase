mod event;
mod event_handler;
mod http;
mod migration;
mod model;

use dirtybase_contract::prelude::*;

pub struct Extension;

#[dirtybase_contract::async_trait]
impl dirtybase_contract::ExtensionSetup for Extension {
    async fn setup(&self, _config: &DirtyConfig) {
        event_handler::setup().await;
    }

    fn migrations(&self) -> dirtybase_contract::ExtensionMigrations {
        migration::setup()
    }

    fn register_routes(
        &self,
        mut manager: RouterManager,
        _middleware: &WebMiddlewareManager,
    ) -> RouterManager {
        // manager.general(None, |router| {
        //     router.get("/be-awesome", || async { "Always" }, "awesomeness")
        // });

        manager
    }
}
