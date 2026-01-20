use dirtybase_contract::prelude::ContextResourceManager;

use crate::{CronJobManager, CronJobRegisterer};

pub async fn register_resource_manager() {
    // dummy job
    CronJobRegisterer::register("foo::job", |ctx| {
        Box::pin(async move {
            tracing::debug!("{} is running but does nothing", ctx.id());
        })
    })
    .await;

    ContextResourceManager::<CronJobManager>::register(
        |_context| {
            Box::pin(async move {
                let name = "global"; // Cron job is always on the global contenxt
                let duration = 0; // Run forever;
                Ok((name, duration).into())
            })
        },
        |_context| {
            Box::pin(async move {
                //
                Ok(CronJobManager::new())
            })
        },
        |manager| {
            Box::pin(async move {
                //
                manager.end().await;
            })
        },
    )
    .await;
}
