use crate::app::Config;

// TODO: Handle redis cluster mode

pub async fn make_redis_client(config: &Config) {
    // if busybody::helpers::service_container()
    //     .get::<redis::Client>()
    //     .is_none()
    //     && !config.redis_connection().is_empty()
    // {
    //     let connection = config.redis_connection();

    //     let client = redis::Client::open(connection.as_ref()).unwrap();
    //     busybody::helpers::service_container().set(client);
    // }
}

pub async fn get_redis_client() -> redis::aio::Connection {
    if let Some(client) = busybody::helpers::service_container().get::<redis::Client>() {
        return client.get_async_connection().await.unwrap();
    } else {
        panic!("Could not get redis client");
    }
}
