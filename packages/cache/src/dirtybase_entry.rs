mod migration;
use dirtybase_contract::{ExtensionMigrations, ExtensionSetup, app_contract::Context};

use crate::config::CacheConfig;

#[derive(Debug, Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, context: &Context) {
        _ = Self::config_from_ctx(context).await;
        super::setup(context).await;
    }

    async fn migrations(&self, _context: &Context) -> Option<ExtensionMigrations> {
        migration::setup()
    }
}

impl Extension {
    pub async fn config_from_ctx(ctx: &Context) -> Result<CacheConfig, anyhow::Error> {
        ctx.get_config_once::<CacheConfig>("cache").await
    }
}
