use dirtybase_config::{CurrentEnvironment, DirtyConfig, APP_NAME_KEY, ENVIRONMENT_KEY};
use std::env;

fn main() {
    let app_name = "Test app";
    let environment: String = CurrentEnvironment::Staging.into();

    env::set_var(APP_NAME_KEY, app_name);
    env::set_var(ENVIRONMENT_KEY, environment);

    let config = DirtyConfig::new_skip();

    println!("name: {}", config.app_name());
    println!("environment: {:?}", config.current_env());
}
