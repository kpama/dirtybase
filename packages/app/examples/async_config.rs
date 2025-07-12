use dirtybase_contract::config_contract::DirtyConfig;

#[tokio::main]
async fn main() {
    let dty_config = DirtyConfig::new();

    let config_json = serde_json::to_string(&dty_config);
    println!("config: {config_json:#?}");
}
