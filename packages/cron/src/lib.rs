pub(crate) mod command_handler;
pub mod config;
mod dirtybase_entry;
pub mod event;
mod job;
mod job_context;
mod manager;

use busstop::DispatchableCommand;
use command_handler::CronJobCommandHandler;
use config::CronConfig;
use dirtybase_contract::app::Context;
pub use dirtybase_entry::*;
pub use job::*;
pub use job_context::*;
pub use manager::*;

pub mod prelude {
    pub use busstop::*;
}

pub async fn start() {
    CronJob::command_handler::<CronJobCommandHandler>().await;
}

pub async fn setup(context: &Context) {
    let config = context
        .get_config::<CronConfig>("dirtybase::cron")
        .await
        .expect("could not get cron configuration");
    setup_using(config).await;
}

pub async fn setup_using(config: CronConfig) -> JobManager {
    if config.enable() {
        start().await;
    }
    JobManager::new(config)
}
