mod cron_job_manager;
mod cron_job_registerer;
mod dirtybase_entry;
mod job;
mod job_context;
mod job_id;

pub(crate) mod cli;
pub(crate) mod command_handler;

pub mod config;
pub mod event;

use busstop::DispatchableCommand;
use command_handler::CronJobCommandHandler;
use config::CronConfig;
use dirtybase_contract::app_contract::Context;

pub use cron_job_manager::*;
pub use cron_job_registerer::*;
pub use dirtybase_entry::*;
pub use job::*;
pub use job_context::*;
pub use job_id::*;

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

pub async fn setup_using(config: CronConfig) -> CronJobManager {
    if config.enable() {
        start().await;
    }
    CronJobManager::new(config)
}
