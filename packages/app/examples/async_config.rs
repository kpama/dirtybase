use dirtybase_contract::config::DirtyConfig;

#[tokio::main]
async fn main() {
    let dty_config = DirtyConfig::new();

    let config_json = serde_json::to_string(&dty_config);
    println!("config: {:#?}", config_json);
}
