use dirtybase_contract::{
    app_contract::ContextResourceManager, db_contract::base::manager::Manager,
    prelude::ResourceManager,
};

use crate::{config::DbConfig, pool_manager_resolver::DbPoolManagerResolver};

pub(crate) async fn register_resource_manager() {
    super::setup_pool_resolvers().await;

    ContextResourceManager::<Manager>::register(
        |context| {
            Box::pin(async move {
                let config = context
                    .get_config_once::<DbConfig>("database")
                    .await
                    .unwrap();
                let timeout = config.idle_timeout();

                let name = context.tenant().await.unwrap_or_default().id().to_string();
                ResourceManager::new(&name, timeout)
            })
        },
        |context| {
            Box::pin(async move {
                let config = context.get::<DbConfig>().await.unwrap();
                let config = config.clone();
                let default_set = config
                    .default_set()
                    .expect("could not get default db config set");
                DbPoolManagerResolver::new(context, default_set)
                    .get_manager()
                    .await
            })
        },
        move |manager| {
            Box::pin(async move {
                tracing::debug!("closing {} pool", manager.db_kind().as_str(),);
                manager.close().await;
            })
        },
    )
    .await;
}
