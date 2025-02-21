use dirtybase_contract::config::DirtyConfig;
use dirtybase_db::config::BaseConfig;

#[tokio::main]
async fn main() {
    let base = DirtyConfig::default();
    let config = BaseConfig::set_from(&base).await;

    println!("db config: {:#?}", &config);
}
