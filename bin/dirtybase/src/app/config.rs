#![allow(dead_code)]

use dirtybase_db::entity::user::hash_password;
use std::env;

use super::DirtyBase;

/// Loads configuration from .env files.
/// Multiple .env files are check in the following order
///  - .env.defaults
///  - .env
///  - .env.dev
///  - .env.prod
/// Values are merged from these files
fn load_dot_env() {
    if env::var("DTY_ENV").is_ok() {
        return;
    }
    let _ = dotenvy::from_filename(".env.defaults");
    let _ = dotenvy::from_filename_override(".env");
    let _ = dotenvy::from_filename_override(".env.dev");
    let _ = dotenvy::from_filename_override(".env.prod");
}

#[derive(Debug, PartialEq, Clone)]
pub enum Environment {
    Production,
    Development,
}

impl Default for Environment {
    fn default() -> Self {
        Self::Development
    }
}

impl Environment {
    pub fn is_prod(&self) -> bool {
        *self == Self::Production
    }

    pub fn is_dev(&self) -> bool {
        *self == Self::Development
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    app_name: String,
    db_connection: String,
    max_db_pool: u32,
    secret: String,
    admin_user: String,
    admin_email: String,
    admin_password: String,
    web_port: u16,
    web_ip_address: String,
    environment: Environment,
    redis_connection: String,
}

impl Default for Config {
    fn default() -> Self {
        load_dot_env();
        let db_connection = env::var("DTY_DATABASE").unwrap_or_default();
        let redis_connection = env::var("DTY_REDIS").unwrap_or_default();
        let max_db_pool: u32 = if let Ok(max) = env::var("DTY_DATABASE_MAX_POOL_CONNECTION") {
            max.parse().unwrap_or(5)
        } else {
            5
        };
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
        let environment = match env::var("DTY_ENV").unwrap_or_default().as_str() {
            "dev" | "development" => Environment::Development,
            "prod" | "production" => Environment::Production,
            _ => Environment::Development,
        };

        Self {
            app_name,
            db_connection,
            max_db_pool,
            secret,
            admin_user,
            admin_email,
            admin_password,
            web_port,
            web_ip_address,
            environment,
            redis_connection,
        }
    }
}

impl Config {
    pub fn app_name(&self) -> &String {
        &self.app_name
    }
    pub fn db_connection(&self) -> &String {
        &self.db_connection
    }

    pub fn max_db_pool(&self) -> u32 {
        self.max_db_pool
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

    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    pub fn redis_connection(&self) -> &String {
        &self.redis_connection
    }
}
pub struct ConfigBuilder {
    app_name: Option<String>,
    db_connection: Option<String>,
    max_db_pool: Option<u32>,
    secret: Option<String>,
    admin_user: Option<String>,
    admin_email: Option<String>,
    admin_password: Option<String>,
    web_port: Option<u16>,
    web_ip_address: Option<String>,
    environment: Option<Environment>,
    redis_connection: Option<String>,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self {
            app_name: None,
            db_connection: None,
            max_db_pool: None,
            secret: None,
            admin_user: None,
            admin_email: None,
            admin_password: None,
            web_port: None,
            web_ip_address: None,
            environment: None,
            redis_connection: None,
        }
    }
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    pub fn app_name(mut self, app_name: &str) -> Self {
        self.app_name = Some(app_name.into());
        self
    }

    pub fn environment(mut self, env: Environment) -> Self {
        self.environment = Some(env);
        self
    }

    pub fn db_connection(mut self, db_connection: &str) -> Self {
        self.db_connection = Some(db_connection.into());
        self
    }

    pub fn max_db_pool(mut self, max_db_pool: u32) -> Self {
        self.max_db_pool = Some(max_db_pool);
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

    pub fn redis_connection(mut self, conn: String) -> Self {
        self.redis_connection = Some(conn);
        self
    }

    pub fn build(self) -> Config {
        let mut config = Config::default();

        config.app_name = self.app_name.unwrap_or(config.app_name);
        config.db_connection = self.db_connection.unwrap_or(config.db_connection);
        config.max_db_pool = self.max_db_pool.unwrap_or(config.max_db_pool);
        config.secret = self.secret.unwrap_or(config.secret);
        config.admin_user = self.admin_user.unwrap_or(config.admin_user);
        config.admin_email = self.admin_email.unwrap_or(config.admin_email);
        config.admin_password = self.admin_password.unwrap_or(config.admin_password);
        config.web_ip_address = self.web_ip_address.unwrap_or(config.web_ip_address);
        config.web_port = self.web_port.unwrap_or(config.web_port);
        config.environment = self.environment.unwrap_or(config.environment);
        config.redis_connection = self.redis_connection.unwrap_or(config.redis_connection);

        config
    }
}

#[busybody::async_trait]
impl busybody::Injectable for Config {
    async fn inject(c: &busybody::ServiceContainer) -> Self {
        c.get::<DirtyBase>().unwrap().config()
    }
}
