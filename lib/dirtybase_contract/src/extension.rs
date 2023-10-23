use dirtybase_config::DirtyConfig;

pub type ExtensionMigrations = Vec<Box<dyn super::db::migration::Migration>>;

#[async_trait::async_trait]
pub trait ExtensionSetup: Send + Sync {
    /// Setup the extension
    async fn setup(&self, config: &DirtyConfig);

    fn migrations(&self) -> ExtensionMigrations {
        Vec::new()
    }

    fn id(&self) -> &str {
        std::any::type_name::<Self>()
    }
}
