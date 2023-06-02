#![allow(dead_code)]

use dirtybase_db::entity::user::hash_password;
use std::env;

fn load_dot_env() {
    let _ = dotenv::from_filename(".env.defaults");
    let _ = dotenv::from_filename(".env");
    let _ = dotenv::from_filename(".env.dev");
    let _ = dotenv::from_filename(".env.prod");
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
}

impl Default for Config {
    fn default() -> Self {
        load_dot_env();
        let db_connection = env::var("DTY_DATABASE").unwrap_or_default();
        let max_db_pool: u32 = if let Ok(max) = env::var("DTY_DATABASE_MAX_POOL_CONNECTION") {
            max.parse().unwrap_or(5)
        } else {
            5
        };
        let secret = env::var("DTY_SECRET").unwrap_or_default();
        let admin_user = env::var("DTY_SYS_ADMIN_USERNAME").unwrap_or_default();
        let admin_email = env::var("DTY_SYS_ADMIN_EMAIL").unwrap_or_default();
        let admin_password = env::var("DTY_SYS_ADMIN_PASSWORD").unwrap_or("changeme!!".into());
        let app_name: String = env::var("DTY_APP_NAME").unwrap_or("Default Company".into());

        Self {
            app_name,
            db_connection,
            max_db_pool,
            secret,
            admin_user,
            admin_email,
            admin_password,
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
}
pub struct ConfigBuilder {
    app_name: Option<String>,
    db_connection: Option<String>,
    max_db_pool: Option<u32>,
    secret: Option<String>,
    admin_user: Option<String>,
    admin_email: Option<String>,
    admin_password: Option<String>,
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

    pub fn build(self) -> Config {
        let mut config = Config::default();

        config.app_name = self.app_name.unwrap_or(config.app_name);
        config.db_connection = self.db_connection.unwrap_or(config.db_connection);
        config.max_db_pool = self.max_db_pool.unwrap_or(config.max_db_pool);
        config.secret = self.secret.unwrap_or(config.secret);
        config.admin_user = self.admin_user.unwrap_or(config.admin_user);
        config.admin_email = self.admin_email.unwrap_or(config.admin_email);
        config.admin_password = self.admin_password.unwrap_or(config.admin_password);

        config
    }
}
