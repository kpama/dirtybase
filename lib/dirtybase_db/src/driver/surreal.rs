use std::env;
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, Surreal};

pub mod surreal_schema_manager;
pub type SurrealClient = Surreal<surrealdb::engine::remote::ws::Client>;

pub struct SurrealDbConfig {
    url: String,
    username: String,
    password: String,
    namespace: String,
    database: String,
}

impl SurrealDbConfig {
    pub fn new(url: &str, username: &str, password: &str, namespace: &str, database: &str) -> Self {
        Self {
            url: url.into(),
            username: username.into(),
            password: password.into(),
            namespace: namespace.into(),
            database: database.into(),
        }
    }

    pub fn new_from_env() -> Self {
        let url = env::var("DTY_SURREAL_URL").unwrap_or_default();
        let username = env::var("DTY_SURREAL_USERNAME").unwrap_or_default();
        let password = env::var("DTY_SURREAL_PASSWORD").unwrap_or_default();
        let namespace = env::var("DTY_SURREAL_NAMESPACE").unwrap_or_default();
        let database = env::var("DTY_SURREAL_DATABASE").unwrap_or_default();

        Self::new(&url, &username, &password, &namespace, &database)
    }
}

pub async fn setup(config: SurrealDbConfig) -> SurrealClient {
    let db = Surreal::new::<Ws>(config.url)
        .await
        .expect("could not connect to database");

    db.signin(Root {
        username: &config.username,
        password: &config.password,
    })
    .await
    .unwrap();

    db.use_ns(&config.namespace)
        .use_db(&config.database)
        .await
        .expect("could not use namespace or database");

    db
}
