use std::{env, path::Path};

use config::builder::DefaultState;

use crate::{load_dot_env, CurrentEnvironment};

#[derive(Debug, Clone)]
pub struct DirtyConfig {
    app_name: String,
    current_env: CurrentEnvironment,
    config_dir: String,
}

impl Default for DirtyConfig {
    fn default() -> Self {
        load_dot_env::<&str>(None);

        Self {
            app_name: env::var(crate::APP_NAME_KEY).unwrap_or(crate::APP_DEFAULT_NAME.into()),
            current_env: env::var(crate::ENVIRONMENT_KEY)
                .unwrap_or_default()
                .as_str()
                .into(),
            config_dir: env::var(crate::CONFIG_DIR_KEY).unwrap_or_default(),
        }
    }
}

impl DirtyConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_at_dir<D: AsRef<Path>>(dir: D) -> Self {
        load_dot_env(Some(dir));

        Self { ..Self::default() }
    }

    pub fn new_with(name: &str, current_env: CurrentEnvironment) -> Self {
        env::set_var(super::LOADED_FLAG_KEY, super::LOADED_FLAG_VALUE);
        Self {
            app_name: name.to_string(),
            current_env,
            config_dir: env::var(crate::CONFIG_DIR_KEY).unwrap_or_default(),
        }
    }

    pub fn new_skip() -> Self {
        env::set_var(super::LOADED_FLAG_KEY, super::LOADED_FLAG_VALUE);

        Self { ..Self::default() }
    }

    pub fn app_name(&self) -> &String {
        &self.app_name
    }

    pub fn current_env(&self) -> &CurrentEnvironment {
        &self.current_env
    }

    pub fn builder(&self) -> config::ConfigBuilder<DefaultState> {
        let env = String::from(self.current_env());
        config::Config::builder()
            .set_override(crate::ENVIRONMENT_KEY, env)
            .unwrap()
    }

    pub fn optional_file(
        &self,
        filename: &str,
        dotenv_prefix: Option<&str>,
    ) -> config::ConfigBuilder<DefaultState> {
        let path = Path::new(&self.config_dir).join(filename);

        let mut builder = if let Some(real) = path.to_str() {
            self.builder()
                .add_source(config::File::with_name(real).required(false))
        } else {
            self.builder()
                .add_source(config::File::with_name(filename).required(false))
        };

        if let Some(prefix) = dotenv_prefix {
            builder = builder.add_source(config::Environment::with_prefix(prefix));
        }

        builder
    }

    pub fn load_optional_file(
        &self,
        full_path: &str,
        dotenv_prefix: Option<&str>,
    ) -> config::ConfigBuilder<DefaultState> {
        let mut builder = self
            .builder()
            .add_source(config::File::with_name(full_path).required(false));

        if let Some(prefix) = dotenv_prefix {
            builder = builder.add_source(config::Environment::with_prefix(prefix));
        }

        builder
    }

    pub fn require_file(
        &self,
        filename: &str,
        dotenv_prefix: Option<&str>,
    ) -> config::ConfigBuilder<DefaultState> {
        let path = Path::new(&self.config_dir).join(filename);

        let mut builder = if let Some(real) = path.to_str() {
            self.builder().add_source(config::File::with_name(real))
        } else {
            self.builder().add_source(config::File::with_name(filename))
        };

        if let Some(prefix) = dotenv_prefix {
            builder = builder.add_source(config::Environment::with_prefix(prefix));
        }

        builder
    }

    pub fn load_file(
        &self,
        full_path: &str,
        dotenv_prefix: Option<&str>,
    ) -> config::ConfigBuilder<DefaultState> {
        let mut builder = self
            .builder()
            .add_source(config::File::with_name(full_path));

        if let Some(prefix) = dotenv_prefix {
            builder = builder.add_source(config::Environment::with_prefix(prefix));
        }

        builder
    }
}

#[busybody::async_trait]
impl busybody::Injectable for DirtyConfig {
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
        let config = DirtyConfig::default();

        assert_eq!(
            CurrentEnvironment::Development,
            config.current_env().clone()
        );
    }

    fn test_overriding() {
        reset_env();
        let app_name = "Test app";
        let environment: String = CurrentEnvironment::Development.into();

        env::set_var(crate::APP_NAME_KEY, app_name);
        env::set_var(crate::ENVIRONMENT_KEY, environment);
        env::set_var(crate::LOADED_FLAG_KEY, crate::LOADED_FLAG_VALUE);

        let config = DirtyConfig::new();

        assert_eq!(config.app_name(), app_name);
        assert_eq!(config.current_env(), &CurrentEnvironment::Development);
    }

    fn test_skipping() {
        reset_env();
        let config = DirtyConfig::new_skip();

        assert_eq!(config.app_name(), crate::APP_DEFAULT_NAME);
        assert_eq!(config.current_env(), &CurrentEnvironment::Development);
    }

    fn test_load_from_dir() {
        reset_env();

        let tmp_dir = env::temp_dir();
        let dir = tmp_dir.join("dirty_config");
        let content = "DTY_APP_NAME=\"My Temp App\" \nDTY_APP_ENV=\"prod\" \n";

        _ = std::fs::create_dir(dir.clone());

        match std::fs::write(dir.join(".env"), content.as_bytes()) {
            Ok(_) => {
                let config = DirtyConfig::new_at_dir(env::temp_dir().join("dirty_config"));
                assert_eq!(config.app_name(), "My Temp App");
                assert_eq!(config.current_env(), &CurrentEnvironment::Production);
            }
            Err(e) => panic!("could not write temp file: {}", e.to_string()),
        }
    }
}
