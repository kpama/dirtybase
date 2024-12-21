use busstop::async_trait;

use crate::CronJob;

#[derive(Debug, Default)]
pub(crate) struct CronJobCommandHandler;

#[async_trait]
impl busstop::CommandHandler for CronJobCommandHandler {
    async fn handle_command(
        &self,
        mut dc: busstop::DispatchedCommand,
    ) -> busstop::DispatchedCommand {
        let job = dc.take_command::<CronJob>().unwrap();
        job.spawn().await;

        dc
    }
}
