use dirtybase_contract::config::DirtyConfig;
use dirtybase_db::config::ConnectionConfig;

#[tokio::main]
async fn main() {
    let base = DirtyConfig::default();
    let config = ConnectionConfig::set_from(&base).await;

    println!("db config: {:#?}", &config);
}
