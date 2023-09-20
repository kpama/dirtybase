use dirtybase_config::{Config, Environment, APP_NAME_KEY, ENVIRONMENT_KEY};
use std::env;

fn main() {
    let app_name = "Test app";
    let environment: String = Environment::Staging.into();

    env::set_var(APP_NAME_KEY, app_name);
    env::set_var(ENVIRONMENT_KEY, environment);

    let config = Config::new_skip();

    println!("name: {}", config.app_name());
    println!("environment: {:?}", config.environment());
}
