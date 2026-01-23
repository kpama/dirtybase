use dirtybase_common::anyhow::{self, Context as AnyhowCtx};
use dirtybase_contract::{
    app_contract::Context,
    async_trait,
    config_contract::{ConfigResult, DirtyConfig, TryFromDirtyConfig},
    multitenant_contract::{
        TENANT_ID_HEADER, TENANT_ID_QUERY_STRING, TENANT_TOKEN_HEADER, TENANT_TOKEN_QUERY_STRING,
        TenantIdLocation,
    },
    serde,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultitenantConfig {
    enable: bool,
    id_location: TenantIdLocation,
    db_config_set: String,
    storage: String,
    header_key: String,
    token_header_key: String,
    query_key: String,
    token_query_key: String,
    tenant_require: bool,
}

impl Default for MultitenantConfig {
    fn default() -> Self {
        Self {
            enable: Default::default(),
            id_location: Default::default(),
            db_config_set: Default::default(),
            storage: Default::default(),
            header_key: TENANT_ID_HEADER.to_string(),
            token_header_key: TENANT_TOKEN_HEADER.to_string(),
            query_key: TENANT_ID_QUERY_STRING.to_string(),
            token_query_key: TENANT_TOKEN_QUERY_STRING.to_string(),
            tenant_require: false,
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

    pub fn header_key(&self) -> &str {
        &self.header_key
    }

    pub fn token_header_key(&self) -> &str {
        &self.token_header_key
    }

    pub fn query_key(&self) -> &str {
        &self.query_key
    }

    pub fn token_query_key(&self) -> &str {
        &self.token_query_key
    }

    pub fn tenant_require(&self) -> bool {
        self.tenant_require
    }
}
