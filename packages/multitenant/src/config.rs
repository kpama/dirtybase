use dirtybase_contract::{
    app::Context,
    async_trait,
    config::{ConfigResult, DirtyConfig, TryFromDirtyConfig},
    multitenant::TenantIdLocation,
    serde,
};

use crate::storage::TenantStorageDriver;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MultitenantConfig {
    enable: bool,
    id_location: TenantIdLocation,
    storage: TenantStorageDriver,
}

impl Default for MultitenantConfig {
    fn default() -> Self {
        Self {
            enable: Default::default(),
            id_location: Default::default(),
            storage: TenantStorageDriver::Dummy,
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
            .expect("could not configure multitenant configuration")
            .try_deserialize()
        {
            Ok(config) => Ok(config),
            Err(_) => Ok(Self::default()),
        }
    }
}

impl MultitenantConfig {
    pub fn is_enabled(&self) -> bool {
        self.enable
    }

    pub fn id_location(&self) -> TenantIdLocation {
        self.id_location.clone()
    }
    pub fn id_location_ref(&self) -> &TenantIdLocation {
        &self.id_location
    }

    // pub async fn from_dirty_config(base: &DirtyConfig) -> Self {
    //     Self::from_config(base).await.unwrap()
    // }
}
