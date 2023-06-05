pub mod base;
pub mod driver;
pub mod entity;

use std::collections::HashMap;

use base::{
    connection::{ConnectionPoolRegisterTrait, ConnectionPoolTrait},
    manager::Manager,
};
pub use dirtybase_db_internal as internal;

#[derive(Debug)]
pub struct ConnectionPoolManager {
    connections: HashMap<String, Box<dyn ConnectionPoolTrait>>,
    default: String,
}

impl ConnectionPoolManager {
    pub async fn new(
        list: Vec<Box<dyn ConnectionPoolRegisterTrait>>,
        default_pool: &str,
        conn_str: &str,
        max: u32,
    ) -> Self {
        let mut connections = HashMap::new();
        let mut default = "".into();

        for (index, entry) in list.iter().enumerate() {
            if let Some(connection_pool) = entry.register(conn_str, max).await {
                let id = connection_pool.id();
                if index == 0 {
                    default = id.clone();
                }
                if &id == default_pool {
                    default = id.clone();
                }

                connections.insert(id, connection_pool);
            }
        }

        Self {
            connections: connections,
            default,
        }
    }

    pub fn default_schema_manager(&self) -> Result<Manager, anyhow::Error> {
        self.schema_manger(&self.default)
    }

    pub fn schema_manger(&self, id: &str) -> Result<Manager, anyhow::Error> {
        match self.connections.get(id) {
            Some(conn_pool) => Ok(Manager::new(conn_pool.schema_manger())),
            None => Err(anyhow::anyhow!("Could not find connection pool: {:?}", id)),
        }
    }
}
