use std::time::Duration;

use tracing::Level;

#[tokio::main]
async fn main() {
    // for logging purposes
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .try_init()
        .expect("could not setup tracing");

    // 1. Initiate the job dispatcher
    dirtybase_cron::start().await;

    // 2. Register a job
    let id = "example::hi".try_into().unwrap();
    let ctx_result = dirtybase_cron::CronJob::register(
        "every 5 seconds",
        |_ctx| {
            Box::pin(async {
                println!("hi from 5 seconds job");
            })
        },
        id,
    )
    .await;

    // 3
    let id = "example::hi2".try_into().unwrap();
    let _ctx = dirtybase_cron::CronJob::register(
        "0/10 * * * * ? *",
        |_ctx| {
            Box::pin(async {
                println!("hi from 10 seconds job");
            })
        },
        id,
    )
    .await;

    if let Ok(ctx) = ctx_result {
        for _ in 0..20 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            _ = ctx.send(dirtybase_cron::event::CronJobCommand::Run).await;
        }
        _ = ctx.send(dirtybase_cron::event::CronJobCommand::Stop).await;
        tokio::time::sleep(Duration::from_secs(6)).await;
        ctx.send(dirtybase_cron::event::CronJobCommand::Run).await;
    }

    // 4. Wait for 60 seconds before completing
    tokio::time::sleep(Duration::from_secs(60)).await;
    println!("completed");
}
