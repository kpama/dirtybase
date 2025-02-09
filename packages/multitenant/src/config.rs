use dirtybase_contract::{config::DirtyConfig, multitenant::TenantIdLocation, serde};

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
            storage: TenantStorageDriver::Memory,
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
}

impl From<&DirtyConfig> for MultitenantConfig {
    fn from(base: &DirtyConfig) -> Self {
        match base
            .optional_file("multitenant.toml", Some("DTY_MULTITENANT"))
            .build()
            .unwrap()
            .try_deserialize()
        {
            Ok(config) => config,
            Err(_) => Self::default(),
        }
    }
}
