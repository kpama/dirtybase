use dirtybase_config::DirtyConfig;

use crate::http::RouterManager;

pub type ExtensionMigrations = Vec<Box<dyn super::db::migration::Migration>>;

#[async_trait::async_trait]
pub trait ExtensionSetup: Send + Sync {
    /// Setup the extension
    async fn setup(&self, config: &DirtyConfig);

    async fn shutdown(&self) {
        // logic to run when the server is shutting down
    }

    /// Register HTTP routes
    fn register_routes(&self, manager: RouterManager) -> RouterManager {
        manager
    }

    fn migrations(&self) -> ExtensionMigrations {
        Vec::new()
    }

    fn id(&self) -> &str {
        std::any::type_name::<Self>()
    }
}
