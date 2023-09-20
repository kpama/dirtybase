use dirtybase_config::Config;

fn main() {
    let config = Config::new_skip();

    println!("default app name: {}", config.app_name());
    println!("default environment: {:?}", config.environment());
}
