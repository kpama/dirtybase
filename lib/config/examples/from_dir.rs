use dirtybase_config::DirtyConfig;
use std::env;

fn main() {
    let tmp_dir = env::temp_dir();
    let dir = tmp_dir.join("dirty_config");
    let content = "DTY_APP_NAME=\"My Awesome App\" \nDTY_ENV=\"prod\" \n";

    _ = std::fs::create_dir(dir.clone());

    match std::fs::write(dir.join(".env"), content.as_bytes()) {
        Ok(_) => {
            let config = DirtyConfig::new_at_dir(env::temp_dir().join("dirty_config"));

            println!("app name: {}", config.app_name());
            println!("environment: {:?}", config.current_env());
        }
        Err(e) => panic!("could not write temp file: {}", e),
    }
}
