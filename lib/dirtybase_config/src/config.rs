use std::{env, path::Path};

use crate::{load_dot_env, Environment};

#[derive(Debug, Clone)]
pub struct Config {
    app_name: String,
    environment: Environment,
}

impl Default for Config {
    fn default() -> Self {
        load_dot_env::<&str>(None);

        Self {
            app_name: env::var(crate::APP_NAME_KEY).unwrap_or(crate::APP_DEFAULT_NAME.into()),
            environment: env::var(crate::ENVIRONMENT_KEY)
                .unwrap_or_default()
                .as_str()
                .into(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_at_dir<D: AsRef<Path>>(dir: D) -> Self {
        load_dot_env(Some(dir));

        Self { ..Self::default() }
    }

    pub fn new_with(name: &str, environment: Environment) -> Self {
        env::set_var(super::LOADED_FLAG_KEY, super::LOADED_FLAG_VALUE);
        Self {
            app_name: name.to_string(),
            environment,
        }
    }

    pub fn new_skip() -> Self {
        env::set_var(super::LOADED_FLAG_KEY, super::LOADED_FLAG_VALUE);

        Self { ..Self::default() }
    }

    pub fn app_name(&self) -> &String {
        &self.app_name
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }
}

#[busybody::async_trait]
impl busybody::Injectable for Config {
    async fn inject(container: &busybody::ServiceContainer) -> Self {
        match container.get_type() {
            Some(config) => config,
            None => {
                container.set_type(Self::default());
                container.get_type().unwrap()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn reset_env() {
        env::remove_var(crate::APP_NAME_KEY);
        env::remove_var(crate::ENVIRONMENT_KEY);
        env::remove_var(crate::LOADED_FLAG_KEY);
    }

    #[test]
    fn rust_test() {
        test_default();
        test_overriding();
        test_skipping();
        test_load_from_dir()
    }

    fn test_default() {
        reset_env();
        let config = Config::default();

        assert_eq!(Environment::Development, config.environment().clone());
    }

    fn test_overriding() {
        reset_env();
        let app_name = "Test app";
        let environment: String = Environment::Development.into();

        env::set_var(crate::APP_NAME_KEY, app_name);
        env::set_var(crate::ENVIRONMENT_KEY, environment);
        env::set_var(crate::LOADED_FLAG_KEY, crate::LOADED_FLAG_VALUE);

        let config = Config::new();

        assert_eq!(config.app_name(), app_name);
        assert_eq!(config.environment(), &Environment::Development);
    }

    fn test_skipping() {
        reset_env();
        let config = Config::new_skip();

        assert_eq!(config.app_name(), crate::APP_DEFAULT_NAME);
        assert_eq!(config.environment(), &Environment::Development);
    }

    fn test_load_from_dir() {
        reset_env();

        let tmp_dir = env::temp_dir();
        let dir = tmp_dir.join("dirty_config");
        let content = "DTY_APP_NAME=\"My Temp App\" \nDTY_ENV=\"prod\" \n";

        _ = std::fs::create_dir(dir.clone());

        match std::fs::write(dir.join(".env"), content.as_bytes()) {
            Ok(_) => {
                let config = Config::new_at_dir(env::temp_dir().join("dirty_config"));
                assert_eq!(config.app_name(), "My Temp App");
                assert_eq!(config.environment(), &Environment::Production);
            }
            Err(e) => panic!("could not write temp file: {}", e.to_string()),
        }
    }
}
