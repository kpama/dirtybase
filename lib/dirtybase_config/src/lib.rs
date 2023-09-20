use std::{
    env,
    path::{Path, PathBuf},
};

mod config;

pub use config::Config;

/// Loads configuration from .env files.
/// Multiple .env files are check in the following order
///  - .env.defaults
///  - .env
///  - .env.dev
///  - .env.stage
///  - .env.prod
/// Values are merged from these files
///

pub const APP_NAME_KEY: &str = "DTY_APP_NAME";
pub const APP_DEFAULT_NAME: &str = "A Dirty App";
pub const ENVIRONMENT_KEY: &str = "DTY_ENV";

pub(crate) const LOADED_FLAG_KEY: &str = "DTY_ENV_LOADED";
pub(crate) const LOADED_FLAG_VALUE: &str = "DTY_YES";

fn load_dot_env<P: AsRef<Path>>(mut dir: Option<P>) {
    if env::var(LOADED_FLAG_KEY).is_ok() {
        return;
    }

    let path: PathBuf = if let Some(p) = dir.take() {
        p.as_ref().join("")
    } else {
        PathBuf::new().join("./")
    };

    if !path.is_dir() {
        panic!("Directory to find .env files does not exist");
    }

    let _ = dotenvy::from_filename(path.join(".env.defaults"));
    let _ = dotenvy::from_filename_override(path.join(".env"));
    let _ = dotenvy::from_filename_override(path.join(".env.dev"));
    let _ = dotenvy::from_filename_override(path.join(".env.stage"));
    let _ = dotenvy::from_filename_override(path.join(".env.prod"));

    env::set_var(LOADED_FLAG_KEY, LOADED_FLAG_VALUE);
}

#[derive(Debug, PartialEq, Clone)]
pub enum Environment {
    Production,
    Staging,
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

    pub fn is_staging(&self) -> bool {
        *self == Self::Staging
    }

    pub fn is_dev(&self) -> bool {
        *self == Self::Development
    }
}

impl From<Environment> for String {
    fn from(value: Environment) -> Self {
        match value {
            Environment::Development => "dev".to_string(),
            Environment::Production => "prod".to_string(),
            Environment::Staging => "staging".to_string(),
        }
    }
}

impl From<String> for Environment {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl From<&str> for Environment {
    fn from(value: &str) -> Self {
        match value {
            "dev" | "development" => Environment::Development,
            "prod" | "production" => Environment::Production,
            "stage" | "staging" => Environment::Staging,
            _ => Environment::Development,
        }
    }
}
