use dirtybase_common::anyhow::{self, Context as AnyhowCtx};
use dirtybase_contract::{
    app_contract::Context,
    async_trait,
    config_contract::{ConfigResult, DirtyConfig, TryFromDirtyConfig},
    multitenant_contract::TenantIdLocation,
    serde,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultitenantConfig {
    enable: bool,
    id_location: TenantIdLocation,
    db_config_set: String,
    storage: String,
}

impl Default for MultitenantConfig {
    fn default() -> Self {
        Self {
            enable: Default::default(),
            id_location: Default::default(),
            db_config_set: Default::default(),
            storage: Default::default(),
        }
    }
}

#[async_trait]
impl TryFromDirtyConfig for MultitenantConfig {
    type Returns = Self;
    async fn from_config(config: &DirtyConfig, _ctx: &Context) -> ConfigResult<Self::Returns> {
        match config
            .optional_file("multitenant.toml", Some("DTY_MULTITENANT"))
            .build()
            .await
            .context("could not configure multitenant configuration")?
            .try_deserialize()
        {
            Ok(config) => Ok(config),
            Err(e) => Err(anyhow::anyhow!("could not load multitenant config: {}", e)),
        }
    }
}

impl MultitenantConfig {
    pub fn is_enabled(&self) -> bool {
        self.enable
    }

    pub fn id_location(&self) -> &TenantIdLocation {
        &self.id_location
    }

    pub fn db_config_set(&self) -> &str {
        &self.db_config_set
    }

    pub fn storage(&self) -> &str {
        &self.storage
    }
}
