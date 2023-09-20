use dirtybase_config::Config;

fn main() {
    let config = Config::default();

    println!("app name: {:#?}", config.app_name());
    println!("environment: {:#?}", config.environment());
}
