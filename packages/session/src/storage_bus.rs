use dirtybase_contract::session::SessionStorageProviderService;
use dirtybase_cron::prelude::DispatchableQuery;

use crate::SessionConfig;

pub type MakeSessionStorageResult = Result<SessionStorageProviderService, anyhow::Error>;
pub struct MakeSessionStorageCommand {
    config: SessionConfig,
}

impl MakeSessionStorageCommand {
    pub fn new(config: SessionConfig) -> Self {
        Self { config }
    }

    pub fn config_ref(&self) -> &SessionConfig {
        &self.config
    }

    pub fn config(&self) -> SessionConfig {
        self.config.clone()
    }
}

#[async_trait::async_trait]
impl DispatchableQuery for MakeSessionStorageCommand {}
