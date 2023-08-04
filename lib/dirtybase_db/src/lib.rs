use base::{
    connection::{ConnectionPoolRegisterTrait, ConnectionPoolTrait},
    manager::Manager,
    schema::DatabaseKind,
};
use std::collections::HashMap;

pub mod base;
pub mod driver;
pub mod entity;
pub mod event;

pub use dirtybase_db_macro as macros;
pub use dirtybase_db_types;

#[derive(Debug)]
pub struct ConnectionPoolManager {
    connections: HashMap<DatabaseKind, Box<dyn ConnectionPoolTrait>>,
    default_pool: DatabaseKind,
}

impl ConnectionPoolManager {
    pub async fn new(
        list: Vec<Box<dyn ConnectionPoolRegisterTrait>>,
        default_pool: DatabaseKind,
        conn_str: &str,
        max: u32,
    ) -> Self {
        let mut connections = HashMap::new();
        // let mut default = default_pool

        for entry in list.into_iter() {
            if let Some(connection_pool) = entry.register(conn_str, max).await {
                let id = connection_pool.id();
                // if index == 0 {
                //     default = id;
                // }
                // if id == default_pool {
                //     default = id;
                // }
                connections.insert(id, connection_pool);
            }
        }

        Self {
            connections,
            default_pool,
        }
    }

    pub fn default_schema_manager(&self) -> Result<Manager, anyhow::Error> {
        self.schema_manger(&self.default_pool)
    }

    pub fn schema_manger(&self, id: &DatabaseKind) -> Result<Manager, anyhow::Error> {
        match self.connections.get(id) {
            Some(conn_pool) => Ok(Manager::new(conn_pool.schema_manger())),
            None => Err(anyhow::anyhow!("Could not find connection pool: {:?}", id)),
        }
    }
}
