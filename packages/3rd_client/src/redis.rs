use anyhow::anyhow;
pub use deadpool_redis;

pub use deadpool_redis::Connection as RedisConnection;

pub const DTY_REDIS_URL_KEY: &str = "DTY_REDIS";

// TODO: Handle redis cluster mode

pub async fn init() {
    if busybody::helpers::service_container()
        .get::<redis::Client>()
        .await
        .is_none()
    {
        let connection = std::env::var(DTY_REDIS_URL_KEY).unwrap_or("redis://redisdb/".into());

        match redis::Client::open(connection.as_ref()) {
            Ok(client) => {
                busybody::helpers::service_container().set(client).await;
            }
            Err(e) => {
                log::error!("error creating redis client: {:?}", e.to_string());
                panic!("{:?}", e);
            }
        }
    }
}

pub async fn get_client() -> Result<redis::aio::MultiplexedConnection, anyhow::Error> {
    if let Some(client) = busybody::helpers::service_container()
        .get::<redis::Client>()
        .await
    {
        Ok(client.get_multiplexed_async_connection().await.unwrap())
    } else {
        log::error!("Could not get redis client");
        Err(anyhow!("Could not get redis client"))
    }
}
