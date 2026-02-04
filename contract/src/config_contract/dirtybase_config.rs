use std::{env, path::Path, sync::Arc};

use base64ct::Encoding;
use config::{builder::AsyncState, ConfigBuilder, Environment};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{
    load_dot_env, CurrentEnvironment, APP_DEFAULT_NAME, APP_NAME_KEY, CONFIG_DIR_KEY,
    ENVIRONMENT_KEY,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirtyConfig {
    app_name: Arc<String>,
    current_env: CurrentEnvironment,
    config_dir: Arc<String>,
}

impl Default for DirtyConfig {
    fn default() -> Self {
        load_dot_env::<&str>(None);

        Self {
            app_name: env::var(APP_NAME_KEY)
                .unwrap_or(APP_DEFAULT_NAME.into())
                .into(),
            current_env: env::var(ENVIRONMENT_KEY)
                .unwrap_or_default()
                .as_str()
                .into(),
            config_dir: env::var(CONFIG_DIR_KEY).unwrap_or_default().into(),
        }
    }
}

impl DirtyConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_at_dir<D: AsRef<Path>>(dir: D) -> Self {
        let path = dir.as_ref().join("");
        load_dot_env(Some(dir));

        let p = path.to_str().to_owned().unwrap_or_default().to_string();
        env::set_var(CONFIG_DIR_KEY, p);

        Self { ..Self::default() }
    }

    pub fn new_with(name: &str, current_env: CurrentEnvironment) -> Self {
        env::set_var(super::LOADED_FLAG_KEY, super::LOADED_FLAG_VALUE);
        Self {
            app_name: Arc::new(name.to_string()),
            current_env,
            config_dir: Arc::new(env::var(CONFIG_DIR_KEY).unwrap_or_default()),
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

    pub fn builder(&self) -> config::ConfigBuilder<AsyncState> {
        let env = String::from(self.current_env());
        ConfigBuilder::<AsyncState>::default()
            .set_override(ENVIRONMENT_KEY, env)
            .unwrap()
    }

    pub fn dotenv_prefix(&self, prefix: &str) -> config::ConfigBuilder<AsyncState> {
        self.builder().add_source(self.build_env(prefix))
    }

    pub fn optional_file(
        &self,
        filename: &str,
        dotenv_prefix: Option<&str>,
    ) -> config::ConfigBuilder<AsyncState> {
        self.load_optional_file(filename, dotenv_prefix)
    }

    pub fn load_optional_file_fn<F>(
        &self,
        filename: &str,
        dotenv_prefix: Option<&str>,
        mut env_callback: F,
    ) -> config::ConfigBuilder<AsyncState>
    where
        F: FnMut(Environment) -> Environment,
    {
        let path = Path::new(self.config_dir.as_str()).join(filename);

        let mut builder = if let Some(full_path) = path.to_str() {
            self.builder()
                .add_source(config::File::with_name(full_path).required(false))
        } else {
            self.builder()
                .add_source(config::File::with_name(filename).required(false))
        };

        builder = self.append_file(builder, filename, false);

        if let Some(prefix) = dotenv_prefix {
            builder = builder.add_source(env_callback(self.build_env(prefix)));
        }

        builder
    }
    pub fn load_optional_file(
        &self,
        filename: &str,
        dotenv_prefix: Option<&str>,
    ) -> config::ConfigBuilder<AsyncState> {
        self.load_optional_file_fn(filename, dotenv_prefix, |ev| ev)
    }

    pub fn require_file(
        &self,
        filename: &str,
        dotenv_prefix: Option<&str>,
    ) -> config::ConfigBuilder<AsyncState> {
        let path = Path::new(self.config_dir.as_str()).join(filename);

        let mut builder = if let Some(real) = path.to_str() {
            self.builder().add_source(config::File::with_name(real))
        } else {
            self.builder().add_source(config::File::with_name(filename))
        };

        builder = self.append_file(builder, filename, true);

        if let Some(prefix) = dotenv_prefix {
            builder = builder.add_source(self.build_env(prefix));
        }

        builder
    }

    pub fn load_file(
        &self,
        full_path: &str,
        dotenv_prefix: Option<&str>,
    ) -> config::ConfigBuilder<AsyncState> {
        let mut builder = self
            .builder()
            .add_source(config::File::with_name(full_path));

        if let Some(prefix) = dotenv_prefix {
            builder = builder.add_source(self.build_env(prefix));
        }

        builder
    }

    fn append_file(
        &self,
        mut builder: config::ConfigBuilder<AsyncState>,
        filename: &str,
        required: bool,
    ) -> config::ConfigBuilder<AsyncState> {
        let env_version = [".", "_prod.", "_stage.", "_dev."];
        for name in env_version {
            let new_file_name = filename.replacen('.', name, 1);
            let path = Path::new(self.config_dir.as_str()).join(new_file_name);

            if let Some(full_path) = path.to_str() {
                builder = builder.add_source(config::File::with_name(full_path).required(required));
            }
        }
        builder
    }

    fn build_env(&self, prefix: &str) -> config::Environment {
        config::Environment::with_prefix(prefix).try_parsing(true)
    }
}

/// A deserializer function that will return `Vec<String>`
///
/// Use this function on an attribute where the raw value is a string
/// that should be split at "," and the pieces return as a `Vec<String>`
///  apply the function as the deserializer as `#[serde(deserialize_with = "field_to_array")]`
pub fn field_to_array<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = serde::de::Deserialize::deserialize(deserializer).unwrap_or_default();

    Ok(s.split(',')
        .map(|v| v.trim().to_string())
        .collect::<Vec<String>>())
}

/// Same as `field_to_array` but an empty string will case a return of `None`
pub fn field_to_option_array<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = serde::de::Deserialize::deserialize(deserializer).unwrap_or_default();
    if s.is_empty() {
        return Ok(None);
    }

    Ok(Some(
        s.split(',')
            .map(|v| v.trim().to_string())
            .collect::<Vec<String>>(),
    ))
}

pub fn field_to_vec_u8<'de, D>(deserializer: D) -> Result<Arc<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = serde::de::Deserialize::deserialize(deserializer).unwrap_or_default();
    if s.starts_with("base64:") {
        Ok(Arc::new(
            base64ct::Base64::decode_vec(&s.replace("base64:", "")).unwrap_or_default(),
        ))
    } else {
        Ok(Arc::new(hex::decode(s).unwrap_or_default()))
    }
}

pub fn vec_u8_to_field<S>(v: &[u8], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&format!("base64:{}", base64ct::Base64::encode_string(v)))
}

#[cfg(test)]
mod test {
    use crate::config_contract::{LOADED_FLAG_KEY, LOADED_FLAG_VALUE};

    use super::*;

    fn reset_env() {
        env::remove_var(APP_NAME_KEY);
        env::remove_var(ENVIRONMENT_KEY);
        env::remove_var(LOADED_FLAG_KEY);
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

        env::set_var(APP_NAME_KEY, app_name);
        env::set_var(ENVIRONMENT_KEY, environment);
        env::set_var(LOADED_FLAG_KEY, LOADED_FLAG_VALUE);

        let config = DirtyConfig::new();

        assert_eq!(config.app_name(), app_name);
        assert_eq!(config.current_env(), &CurrentEnvironment::Development);
    }

    fn test_skipping() {
        reset_env();
        let config = DirtyConfig::new_skip();

        assert_eq!(config.app_name(), APP_DEFAULT_NAME);
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
            Err(e) => panic!("could not write temp file: {}", e),
        }
    }
}
