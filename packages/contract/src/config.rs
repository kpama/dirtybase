use std::{
    env,
    path::{Path, PathBuf},
};
mod dirtybase_config;

pub use dirtybase_config::*;

pub use config;
use serde::{Deserialize, Serialize};

use crate::app::Context;

/// Loads configuration from .env files.
/// Multiple .env files are check in the following order
///  - .env.defaults
///  - .env.prod
///  - .env.stage
///  - .env
///  - .env.dev
///
/// Values are merged from these files
pub const APP_NAME_KEY: &str = "DTY_APP_NAME";
pub const APP_DEFAULT_NAME: &str = "A Dirty App";
pub const ENVIRONMENT_KEY: &str = "DTY_APP_ENV";
pub const CONFIG_DIR_KEY: &str = "DTY_APP_CONFIG_DIR";

pub(crate) const LOADED_FLAG_KEY: &str = "DTY_ENV_LOADED";
pub(crate) const LOADED_FLAG_VALUE: &str = "DTY_YES";

pub type ConfigResult<C> = Result<C, anyhow::Error>;

#[async_trait::async_trait]
pub trait TryFromDirtyConfig {
    type Returns;
    async fn from_config(config: &DirtyConfig, ctx: &Context) -> ConfigResult<Self::Returns>;
}

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
    let _ = dotenvy::from_filename_override(path.join(".env.prod"));
    let _ = dotenvy::from_filename_override(path.join(".env.stage"));
    let _ = dotenvy::from_filename_override(path.join(".env"));
    let _ = dotenvy::from_filename_override(path.join(".env.dev"));

    env::set_var(LOADED_FLAG_KEY, LOADED_FLAG_VALUE);
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum CurrentEnvironment {
    #[serde(rename = "prod")]
    Production,
    #[serde(rename = "staging")]
    Staging,
    #[serde(rename = "dev")]
    Development,
}

impl Default for CurrentEnvironment {
    fn default() -> Self {
        Self::Development
    }
}

impl CurrentEnvironment {
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

impl From<CurrentEnvironment> for String {
    fn from(value: CurrentEnvironment) -> Self {
        String::from(&value)
    }
}

impl From<&CurrentEnvironment> for String {
    fn from(value: &CurrentEnvironment) -> Self {
        match value {
            CurrentEnvironment::Development => "dev".to_string(),
            CurrentEnvironment::Production => "prod".to_string(),
            CurrentEnvironment::Staging => "staging".to_string(),
        }
    }
}

impl From<String> for CurrentEnvironment {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl From<&str> for CurrentEnvironment {
    fn from(value: &str) -> Self {
        match value {
            "dev" | "development" => CurrentEnvironment::Development,
            "prod" | "production" => CurrentEnvironment::Production,
            "stage" | "staging" => CurrentEnvironment::Staging,
            _ => CurrentEnvironment::Development,
        }
    }
}
