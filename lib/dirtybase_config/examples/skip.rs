use dirtybase_config::DirtyConfig;

fn main() {
    let config = DirtyConfig::new_skip();

    println!("default app name: {}", config.app_name());
    println!("default environment: {:?}", config.current_env());
}
