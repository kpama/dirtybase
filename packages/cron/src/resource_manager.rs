use dirtybase_contract::prelude::ContextResourceManager;

use crate::{CronJobManager, CronJobRegisterer, JobHandlerWrapper};

pub async fn register_resource_manager() {
    // dummy job
    CronJobRegisterer::register("foo::job", |ctx| {
        Box::pin(async move {
            tracing::debug!("{} is running but does nothing", ctx.id());
        })
    })
    .await;

    ContextResourceManager::<CronJobManager>::register(
        |context| {
            Box::pin(async move {
                let name = context
                    .tenant()
                    .await
                    .expect("could not get tenant")
                    .id()
                    .to_string();

                let duration = 0; // if context.is_global() { 0 } else { 5 };
                (name, duration).into()
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
