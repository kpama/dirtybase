use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use busstop::{DispatchableQuery, DispatchedQuery, QueryHandler};

use super::{
    base::{manager::Manager, schema::DatabaseKind},
    config::ConfigSet,
    PoolManagerSet,
};

pub type PoolManagerCommandResult = Result<PoolManagerSet, anyhow::Error>;

#[derive(Debug, Default)]
pub struct MakePoolManagerCommand {
    config_set: ConfigSet,
}

impl MakePoolManagerCommand {
    pub fn config_set_ref(&self) -> &ConfigSet {
        &self.config_set
    }

    pub fn kind(&self) -> DatabaseKind {
        if let Some(config) = self.config_set.values().next() {
            config.kind()
        } else {
            DatabaseKind::Custom("Unknown Kind".to_string())
        }
    }

    pub fn set_result(&self, dispatched: &DispatchedQuery, result: PoolManagerCommandResult) {
        dispatched.set_value(result);
    }

    pub async fn make(config_set: ConfigSet) -> Result<Manager, anyhow::Error> {
        _ = Self::register_soft_query_handler(MakePoolManagerCommandHandler).await;

        let instance = Self {
            config_set: config_set.clone(),
        };
        let kind = instance.kind();

        let result = instance
            .dispatch_query()
            .await
            .take_value::<PoolManagerCommandResult>();

        if let Some(r) = result {
            match *r {
                Ok(c) => {
                    let mut collection = HashMap::new();
                    collection.insert(kind.clone(), c);
                    Ok(Manager::new(Arc::new(collection), kind, config_set))
                }
                Err(e) => Err(e),
            }
        } else {
            Err(anyhow!("error"))
        }
    }

    pub fn make_sync(config_set: ConfigSet) -> Result<Manager, anyhow::Error> {
        tokio::task::block_in_place(move || {
            tokio::runtime::Handle::current().block_on(async move { Self::make(config_set).await })
        })
    }
}

#[busstop::async_trait]
impl DispatchableQuery for MakePoolManagerCommand {}

#[derive(Default)]
struct MakePoolManagerCommandHandler;

#[busstop::async_trait]
impl QueryHandler for MakePoolManagerCommandHandler {
    async fn handle_query(&self, dispatched: DispatchedQuery) -> DispatchedQuery {
        if let Some(query) = dispatched.the_query::<MakePoolManagerCommand>() {
            dispatched.set_value::<PoolManagerCommandResult>(Err(anyhow!(
                "Pool Manager not found: {:?}",
                query.kind()
            )));
        }
        dispatched
    }
}
