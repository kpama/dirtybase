use std::sync::Arc;

use base64ct::Encoding;
use dirtybase_contract::app::Context;
use dirtybase_contract::config::ConfigResult;
use dirtybase_contract::config::DirtyConfig;
use dirtybase_contract::config::TryFromDirtyConfig;
use dirtybase_contract::config::field_to_vec_u8;
use dirtybase_contract::config::vec_u8_to_field;
use serde::Deserializer;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct MiddlewareConfig {
    global: Option<Vec<String>>,
    general_route: Option<Vec<String>>,
    api_route: Option<Vec<String>>,
    insecure_api_route: Option<Vec<String>>,
    admin_route: Option<Vec<String>>,
    dev_route: Option<Vec<String>>,
}

impl MiddlewareConfig {
    pub fn global(&self) -> &Option<Vec<String>> {
        &self.global
    }

    pub fn general_route(&self) -> &Option<Vec<String>> {
        &self.general_route
    }

    pub fn api_route(&self) -> &Option<Vec<String>> {
        &self.api_route
    }

    pub fn insecure_api_route(&self) -> &Option<Vec<String>> {
        &self.insecure_api_route
    }

    pub fn admin_route(&self) -> &Option<Vec<String>> {
        &self.admin_route
    }

    pub fn dev_route(&self) -> &Option<Vec<String>> {
        &self.dev_route
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct ConfigEntry {
    name: String,
    #[serde(
        deserialize_with = "field_to_vec_u8",
        serialize_with = "vec_u8_to_field"
    )]
    key: Arc<Vec<u8>>,
    #[serde(deserialize_with = "field_previous_keys")]
    previous_keys: Option<Arc<Vec<Vec<u8>>>>,
    web_port: u16,
    web_ip_address: String,
    web_enable_api_routes: bool,
    web_enable_insecure_api_routes: bool,
    web_enable_admin_routes: bool,
    web_enable_general_routes: bool,
    web_enable_dev_routes: bool,
    web_api_route_prefix: String,
    web_insecure_api_route_prefix: String,
    web_admin_route_prefix: String,
    web_dev_route_prefix: String,
    #[serde(rename = "web_public_directory")]
    web_public_dir: String,
    web_middleware: MiddlewareConfig,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Config {
    dirty_config: DirtyConfig,
    entry: ConfigEntry,
}

impl Config {
    pub async fn new(config: Option<DirtyConfig>) -> Self {
        let config = config.unwrap_or_default();
        Self::try_from_config(&config)
            .await
            .expect("Could not find application configuration. You need at least a .env file")
    }

    pub async fn try_from_config(config: &DirtyConfig) -> ConfigResult<Self> {
        let builder = config
            .load_optional_file_fn("app.toml", Some("DTY_APP"), |ev| {
                // env entries where the values are Vec<T>
                ev.list_separator(",")
                    .with_list_parse_key("web_middleware.global")
                    .with_list_parse_key("web_middleware.general_route")
                    .with_list_parse_key("web_middleware.api_route")
                    .with_list_parse_key("web_middleware.insecure_api_route")
                    .with_list_parse_key("web_middleware.admin_route")
                    .with_list_parse_key("web_middleware.dev_route")
            })
            .build()
            .await?;

        Ok(Self {
            dirty_config: config.clone(),
            entry: builder.try_deserialize()?,
        })
    }

    pub fn middleware(&self) -> &MiddlewareConfig {
        &self.entry.web_middleware
    }

    pub fn app_name(&self) -> &str {
        self.entry.name.as_str()
    }

    pub fn key(&self) -> Arc<Vec<u8>> {
        self.entry.key.clone()
    }

    pub fn key_ref(&self) -> &[u8] {
        &self.entry.key
    }

    pub fn previous_keys(&self) -> Option<Arc<Vec<Vec<u8>>>> {
        self.entry.previous_keys.clone()
    }

    pub fn previous_keys_ref(&self) -> &Option<Arc<Vec<Vec<u8>>>> {
        &self.entry.previous_keys
    }

    pub fn web_port(&self) -> u16 {
        self.entry.web_port
    }

    pub fn web_ip_address(&self) -> &str {
        self.entry.web_ip_address.as_str()
    }

    pub fn web_enable_api_routes(&self) -> bool {
        self.entry.web_enable_api_routes
    }

    pub fn web_enable_insecure_api_routes(&self) -> bool {
        self.entry.web_enable_insecure_api_routes
    }

    pub fn web_enable_admin_routes(&self) -> bool {
        self.entry.web_enable_admin_routes
    }

    pub fn web_enable_general_routes(&self) -> bool {
        self.entry.web_enable_general_routes
    }

    pub fn web_enable_dev_routes(&self) -> bool {
        self.entry.web_enable_dev_routes
    }

    pub fn web_api_route_prefix(&self) -> &str {
        self.entry.web_api_route_prefix.as_str()
    }

    pub fn web_insecure_api_route_prefix(&self) -> &str {
        self.entry.web_insecure_api_route_prefix.as_str()
    }

    pub fn web_admin_route_prefix(&self) -> &str {
        self.entry.web_admin_route_prefix.as_str()
    }

    pub fn web_dev_route_prefix(&self) -> &str {
        self.entry.web_dev_route_prefix.as_str()
    }

    pub fn web_public_dir(&self) -> &str {
        self.entry.web_public_dir.as_str()
    }

    pub fn environment(&self) -> &dirtybase_contract::config::CurrentEnvironment {
        self.dirty_config.current_env()
    }

    pub fn dirty_config(&self) -> &dirtybase_contract::config::DirtyConfig {
        &self.dirty_config
    }
}

#[derive(Default)]
pub struct ConfigBuilder {
    app_name: Option<String>,
    key: Option<Arc<Vec<u8>>>,
    previous_keys: Option<Arc<Vec<Vec<u8>>>>,
    web_port: Option<u16>,
    web_ip_address: Option<String>,
    web_enable_api_routes: Option<bool>,
    web_enable_insecure_api_routes: Option<bool>,
    web_enable_admin_routes: Option<bool>,
    web_enable_general_routes: Option<bool>,
    web_middleware: Option<MiddlewareConfig>,
    dirty_config: Option<dirtybase_contract::config::DirtyConfig>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    pub fn dirty_config(mut self, config: DirtyConfig) -> Self {
        self.dirty_config = Some(config);
        self
    }

    pub fn app_name(mut self, app_name: &str) -> Self {
        self.app_name = Some(app_name.into());
        self
    }

    pub fn key(mut self, key: Vec<u8>) -> Self {
        self.key = Some(Arc::new(key));
        self
    }

    pub fn previous_keys(mut self, keys: Vec<Vec<u8>>) -> Self {
        self.previous_keys = Some(Arc::new(keys));
        self
    }

    pub fn web_ip_address(mut self, address: &str) -> Self {
        self.web_ip_address = Some(address.into());
        self
    }
    pub fn web_port(mut self, port: u16) -> Self {
        self.web_port = Some(port);
        self
    }

    pub fn web_enable_general_routes(mut self, enable: bool) -> Self {
        self.web_enable_general_routes = Some(enable);
        self
    }

    pub fn web_enable_admin_routes(mut self, enable: bool) -> Self {
        self.web_enable_admin_routes = Some(enable);
        self
    }

    pub fn web_enable_api_routes(mut self, enable: bool) -> Self {
        self.web_enable_api_routes = Some(enable);
        self
    }

    pub fn web_enable_insecure_api_routes(mut self, enable: bool) -> Self {
        self.web_enable_insecure_api_routes = Some(enable);
        self
    }

    pub fn web_middleware(mut self, config: MiddlewareConfig) -> Self {
        self.web_middleware = Some(config);
        self
    }

    pub async fn build(self) -> Config {
        let mut config = Config::new(None).await;

        config.entry.name = self.app_name.unwrap_or(config.entry.name);
        config.entry.key = self.key.unwrap_or(config.entry.key);
        config.entry.web_ip_address = self.web_ip_address.unwrap_or(config.entry.web_ip_address);
        config.entry.web_port = self.web_port.unwrap_or(config.entry.web_port);
        config.entry.web_enable_api_routes = self
            .web_enable_api_routes
            .unwrap_or(config.entry.web_enable_api_routes);
        config.entry.web_enable_insecure_api_routes = self
            .web_enable_insecure_api_routes
            .unwrap_or(config.entry.web_enable_insecure_api_routes);
        config.entry.web_middleware = self.web_middleware.unwrap_or(config.entry.web_middleware);
        config.entry.web_enable_admin_routes = self
            .web_enable_admin_routes
            .unwrap_or(config.entry.web_enable_admin_routes);
        config.entry.web_enable_general_routes = self
            .web_enable_general_routes
            .unwrap_or(config.entry.web_enable_general_routes);

        config.dirty_config = self.dirty_config.unwrap_or(config.dirty_config);

        config
    }
}

pub fn field_previous_keys<'de, D>(deserializer: D) -> Result<Option<Arc<Vec<Vec<u8>>>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = serde::de::Deserialize::deserialize(deserializer).unwrap_or_default();

    if s.trim().is_empty() {
        return Ok(None);
    }

    Ok(Some(Arc::new(
        s.split(',')
            .into_iter()
            .map(|v| {
                let s = v.trim();
                if s.starts_with("base64:") {
                    base64ct::Base64::decode_vec(&s.replace("base64:", "")).unwrap_or_default()
                } else {
                    hex::decode(s).unwrap_or_default()
                }
            })
            .collect::<Vec<Vec<u8>>>(),
    )))
}

#[async_trait::async_trait]
impl TryFromDirtyConfig for Config {
    type Returns = Self;

    async fn from_config(config: &DirtyConfig, _ctx: &Context) -> ConfigResult<Self::Returns> {
        Self::try_from_config(config).await
    }
}
