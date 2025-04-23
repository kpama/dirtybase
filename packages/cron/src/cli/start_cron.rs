use dirtybase_contract::prelude::Context;

use crate::{CronJobManager, config::CronConfig};

pub(super) async fn execute(context: Context) -> Result<(), anyhow::Error> {
    if let Ok(manager) = context.get::<CronJobManager>().await {
        if let Ok(cron_config) = context.get::<CronConfig>().await {
            manager.run(cron_config, context).await;
        }
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                manager.end().await;
                return Ok(())
            },
            _ = terminate => {
                manager.end().await;
                return Ok(())
            },
        }
    }
    Ok(())
}
