pub(crate) mod command_handler;
pub mod config;
mod cron_dirtybase_entry;
pub mod event;
mod job;
mod job_context;
mod manager;

use busstop::DispatchableCommand;
use command_handler::CronJobCommandHandler;
use config::CronConfig;
pub use cron_dirtybase_entry::*;
pub use job::*;
pub use job_context::*;
pub use manager::*;

pub mod prelude {
    pub use busstop::*;
}

pub async fn setup(base_config: &dirtybase_config::DirtyConfig) {
    let config: CronConfig = base_config.into();
    setup_using(config).await;
}

pub async fn setup_using(config: CronConfig) -> JobManager {
    if config.enable() {
        CronJob::command_handler::<CronJobCommandHandler>().await;
    }
    JobManager::new(config)
}
