use dirtybase_3rd_client::redis::get_client;
use redis::AsyncCommands;

#[tokio::main]
async fn main() {
    dirtybase_3rd_client::redis::init().await;
    let mut conn = get_client().await.unwrap();

    _ = conn.set::<&str, &str, ()>("deadpool/test_key", "42").await;

    let value: String = conn.get("deadpool/test_key").await.unwrap();

    println!("value is: {}", &value);
    assert_eq!(value, "42".to_string());
}
