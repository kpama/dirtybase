use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use dirtybase_contract::{
    db_contract::{
        PoolManagerSet,
        base::{manager::Manager, schema::DatabaseKind},
    },
    prelude::Context,
};

use crate::{config::ConfigSet, make_manager};

pub struct DbPoolManagerResolver {
    config: ConfigSet,
    context: Context,
    manager_set: Result<PoolManagerSet, anyhow::Error>,
}

impl DbPoolManagerResolver {
    pub fn new(context: Context, config: ConfigSet) -> Self {
        Self {
            config,
            context,
            manager_set: Err(anyhow!("could not resolve database pool manager set")),
        }
    }
    pub fn config_ref(&self) -> &ConfigSet {
        &self.config
    }

    pub fn config(&self) -> ConfigSet {
        self.config.clone()
    }

    pub fn context_ref(&self) -> &Context {
        &self.context
    }

    pub fn context(&self) -> Context {
        self.context.clone()
    }

    pub fn kind(&self) -> DatabaseKind {
        if let Some(config) = self.config.values().next() {
            config.kind()
        } else {
            "unknown".into()
        }
    }

    pub fn set_pool_manager(&mut self, set: Result<PoolManagerSet, anyhow::Error>) {
        self.manager_set = set;
    }

    pub async fn get_manager(self) -> Result<Manager, anyhow::Error> {
        let kind = self.kind();
        let config_set = self.config();
        let pool_set = Self::get_middleware().await.send(self).await.manager_set?;
        let mut connections = HashMap::new();
        connections.insert(kind.clone(), pool_set);
        Ok(make_manager(connections, kind, &config_set))
    }

    pub fn has_pool_manager(&self) -> bool {
        self.manager_set.is_ok()
    }

    pub async fn register<F, Fut>(name: &str, callback: F)
    where
        F: Clone + Fn(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Self> + Send + 'static,
    {
        let resolvers = Self::get_middleware().await;

        let arc_name = Arc::new(name.to_string());
        resolvers
            .next(move |mut resolver, next| {
                let cb = callback.clone();
                let name = arc_name.clone();
                Box::pin(async move {
                    if resolver.kind().as_str() == *name.as_ref() {
                        resolver = (cb)(resolver).await;
                    }

                    if !resolver.has_pool_manager() {
                        next.call(resolver).await
                    } else {
                        resolver
                    }
                })
            })
            .await;
    }

    async fn get_middleware() -> Arc<simple_middleware::Manager<Self, Self>> {
        if let Some(r) = busybody::helpers::service_container().get().await {
            r
        } else {
            let manager = simple_middleware::Manager::<Self, Self>::last(|resolver, _| {
                Box::pin(async move {
                    //
                    resolver
                })
            })
            .await;
            busybody::helpers::service_container()
                .set(manager)
                .await
                .get()
                .await
                .unwrap() // should never failed as we just registered the instance
        }
    }
}
