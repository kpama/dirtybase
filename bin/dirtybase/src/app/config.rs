#![allow(dead_code)]

use dirtybase_config::DirtyConfig;
use dirtybase_contract::db::entity::user::hash_password;
use std::env;

use super::DirtyBaseApp;

#[derive(Debug, Clone)]
pub struct Config {
    app_name: String,
    secret: String,
    admin_user: String,
    admin_email: String,
    admin_password: String,
    web_port: u16,
    web_ip_address: String,
    dirty_config: dirtybase_config::DirtyConfig,
}

impl Default for Config {
    fn default() -> Self {
        let config = dirtybase_config::DirtyConfig::new();
        Self::new(config)
    }
}

impl Config {
    pub fn new(config: DirtyConfig) -> Self {
        let web_port = if let Ok(p) = env::var("DTY_WEB_PORT") {
            p.parse().unwrap_or(8080)
        } else {
            8080
        };
        let web_ip_address = if let Ok(p) = env::var("DTY_WEB_IP_ADDRESS") {
            p.parse().unwrap_or("127.0.0.1".to_string())
        } else {
            "127.0.0.1".to_owned()
        };
        let secret = env::var("DTY_SECRET").unwrap_or_default();
        let admin_user = env::var("DTY_SYS_ADMIN_USERNAME").unwrap_or_default();
        let admin_email = env::var("DTY_SYS_ADMIN_EMAIL").unwrap_or_default();
        let admin_password = env::var("DTY_SYS_ADMIN_PASSWORD").unwrap_or("changeme!!".into());
        let app_name: String = env::var("DTY_APP_NAME").unwrap_or("Default Company".into());

        Self {
            app_name,
            secret,
            admin_user,
            admin_email,
            admin_password,
            web_port,
            web_ip_address,
            dirty_config: config,
        }
    }
    pub fn app_name(&self) -> &String {
        &self.app_name
    }

    pub fn secret(&self) -> &String {
        &self.secret
    }

    pub fn admin_user(&self) -> &String {
        &self.admin_user
    }

    pub fn admin_email(&self) -> &String {
        &self.admin_email
    }
    pub fn admin_password(&self) -> &String {
        &self.admin_password
    }

    pub fn web_port(&self) -> u16 {
        self.web_port
    }

    pub fn web_ip_address(&self) -> &String {
        &self.web_ip_address
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
    admin_user: Option<String>,
    admin_email: Option<String>,
    admin_password: Option<String>,
    web_port: Option<u16>,
    web_ip_address: Option<String>,
    dirty_config: Option<dirtybase_config::DirtyConfig>,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self {
            app_name: None,
            secret: None,
            admin_user: None,
            admin_email: None,
            admin_password: None,
            web_port: None,
            web_ip_address: None,
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

    pub fn admin_user(mut self, admin_user: &str) -> Self {
        self.admin_user = Some(admin_user.into());
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

    pub fn build(self) -> Config {
        let mut config = Config::default();

        config.app_name = self.app_name.unwrap_or(config.app_name);
        config.secret = self.secret.unwrap_or(config.secret);
        config.admin_user = self.admin_user.unwrap_or(config.admin_user);
        config.admin_email = self.admin_email.unwrap_or(config.admin_email);
        config.admin_password = self.admin_password.unwrap_or(config.admin_password);
        config.web_ip_address = self.web_ip_address.unwrap_or(config.web_ip_address);
        config.web_port = self.web_port.unwrap_or(config.web_port);
        config.dirty_config = self.dirty_config.unwrap_or(config.dirty_config);

        config
    }
}

#[busybody::async_trait]
impl busybody::Injectable for Config {
    async fn inject(c: &busybody::ServiceContainer) -> Self {
        c.get::<DirtyBaseApp>().unwrap().config()
    }
}
