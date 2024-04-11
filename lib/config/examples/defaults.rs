use dirtybase_config::DirtyConfig;

fn main() {
    let config = DirtyConfig::default();

    println!("app name: {:#?}", config.app_name());
    println!("environment: {:#?}", config.current_env());
}
