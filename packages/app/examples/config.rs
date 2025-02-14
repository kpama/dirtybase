use std::collections::HashMap;

use dirtybase_contract::config::DirtyConfig;
use dirtybase_db::{base::schema::ClientType, config::BaseConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    let dty_config = DirtyConfig::new();
    let config = DbConfigurations::new(&dty_config).await;

    println!("{:#?}", config);

    Ok(())
}

#[derive(Debug, serde::Deserialize)]
struct Conf {
    name: String,
    age: i32,
}

#[derive(Debug, serde::Deserialize)]
struct DbConfigurations {
    clients: HashMap<String, BaseConfig>,
}

impl DbConfigurations {
    pub async fn new(config: &DirtyConfig) -> Self {
        let configs = config
            .optional_file("test.toml", Some("DTY_DB"))
            .build()
            .await
            .unwrap();

        dbg!(&configs.get::<HashMap<String, HashMap<ClientType, BaseConfig>>>("clients"));

        // dbg!(&configs);
        Self {
            clients: HashMap::new(),
        }
    }
}
