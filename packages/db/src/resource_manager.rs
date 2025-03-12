use busybody::ServiceContainer;
use dirtybase_contract::{app::ContextResourceManager, db::base::manager::Manager};

use crate::{config::DbConfig, connection_bus::MakePoolManagerCommand};

pub(crate) async fn register_resource_manager(sc: &ServiceContainer) {
    sc.set(ContextResourceManager::<Manager>::new(
        |context| {
            Box::pin(async move {
                let config = context.get_config::<DbConfig>("database").await.unwrap();
                let name = context.tenant().await.unwrap().id().to_string();
                let timeout = config.idle_timeout();
                context.set(config).await;
                (name, timeout)
            })
        },
        |context| {
            Box::pin(async move {
                let config = context.get::<DbConfig>().await.unwrap();
                let config = config.clone();
                let default_set = config
                    .default_set()
                    .expect("could not get default db config set");
                MakePoolManagerCommand::make(default_set)
                    .await
                    .expect("could not create a database manager")
            })
        },
        move |manager| {
            tracing::debug!(
                "closing {} pool, it has been idle for a while",
                manager.db_kind().as_str(),
            );
            Box::pin(async move {
                manager.close().await;
            })
        },
    ))
    .await;
}
