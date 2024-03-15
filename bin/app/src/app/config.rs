use dirtybase_config::DirtyConfig;
use dirtybase_contract::db::entity::user::hash_password;

use super::App;

#[derive(Debug, serde::Deserialize, Clone)]
struct ConfigEntry {
    name: String,
    secret: String,
    sys_admin_username: String,
    sys_admin_email: String,
    sys_admin_password: String,
    web_port: u16,
    web_ip_address: String,
    web_enable_api_routes: bool,
    web_enable_admin_routes: bool,
    web_enable_general_routes: bool,
    #[serde(rename = "web_public_directory")]
    web_public_dir: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    dirty_config: DirtyConfig,
    entry: ConfigEntry,
}

impl Default for Config {
    fn default() -> Self {
        let config = dirtybase_config::DirtyConfig::new();
        Self::new(config)
    }
}

impl Config {
    pub fn new(config: DirtyConfig) -> Self {
        let entry: ConfigEntry = config
            .load_optional_file("app.toml", Some("DTY_APP"))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap();

        Self {
            dirty_config: config,
            entry,
        }
    }

    pub fn app_name(&self) -> &String {
        &self.entry.name
    }

    pub fn secret(&self) -> &String {
        &self.entry.secret
    }

    pub fn admin_username(&self) -> &String {
        &self.entry.sys_admin_username
    }

    pub fn admin_email(&self) -> &String {
        &self.entry.sys_admin_email
    }
    pub fn admin_password(&self) -> &String {
        &self.entry.sys_admin_password
    }

    pub fn web_port(&self) -> u16 {
        self.entry.web_port
    }

    pub fn web_ip_address(&self) -> &String {
        &self.entry.web_ip_address
    }

    pub fn web_enable_api_routes(&self) -> bool {
        self.entry.web_enable_api_routes
    }

    pub fn web_enable_admin_routes(&self) -> bool {
        self.entry.web_enable_admin_routes
    }

    pub fn web_enable_general_routes(&self) -> bool {
        self.entry.web_enable_general_routes
    }
    pub fn web_public_dir(&self) -> &String {
        &self.entry.web_public_dir
    }

    pub fn environment(&self) -> &dirtybase_config::CurrentEnvironment {
        &self.dirty_config.current_env()
    }

    pub fn dirty_config(&self) -> &dirtybase_config::DirtyConfig {
        &self.dirty_config
    }
}

pub struct ConfigBuilder {
    app_name: Option<String>,
    secret: Option<String>,
    admin_username: Option<String>,
    admin_email: Option<String>,
    admin_password: Option<String>,
    web_port: Option<u16>,
    web_ip_address: Option<String>,
    web_enable_api_routes: Option<bool>,
    web_enable_admin_routes: Option<bool>,
    web_enable_general_routes: Option<bool>,
    dirty_config: Option<dirtybase_config::DirtyConfig>,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self {
            app_name: None,
            secret: None,
            admin_username: None,
            admin_email: None,
            admin_password: None,
            web_port: None,
            web_ip_address: None,
            web_enable_api_routes: None,
            web_enable_admin_routes: None,
            web_enable_general_routes: None,
            dirty_config: None,
        }
    }
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

    pub fn secret(mut self, secret: &str) -> Self {
        self.secret = Some(secret.into());
        self
    }

    pub fn admin_username(mut self, admin_user: &str) -> Self {
        self.admin_username = Some(admin_user.into());
        self
    }

    pub fn admin_email(mut self, admin_email: &str) -> Self {
        self.admin_email = Some(admin_email.into());
        self
    }

    pub fn admin_password(mut self, admin_password: &str) -> Self {
        self.admin_password = Some(hash_password(admin_password));
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

    pub fn build(self) -> Config {
        let mut config = Config::default();

        config.entry.name = self.app_name.unwrap_or(config.entry.name);
        config.entry.secret = self.secret.unwrap_or(config.entry.secret);
        config.entry.sys_admin_username = self
            .admin_username
            .unwrap_or(config.entry.sys_admin_username);
        config.entry.sys_admin_email = self.admin_email.unwrap_or(config.entry.sys_admin_email);
        config.entry.sys_admin_password = self
            .admin_password
            .unwrap_or(config.entry.sys_admin_password);
        config.entry.web_ip_address = self.web_ip_address.unwrap_or(config.entry.web_ip_address);
        config.entry.web_port = self.web_port.unwrap_or(config.entry.web_port);
        config.entry.web_enable_api_routes = self
            .web_enable_api_routes
            .unwrap_or(config.entry.web_enable_api_routes);
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

#[busybody::async_trait]
impl busybody::Injectable for Config {
    async fn inject(c: &busybody::ServiceContainer) -> Self {
        c.get::<App>().unwrap().config()
    }
}
