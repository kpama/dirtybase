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
    let _ctx = dirtybase_cron::CronJob::register(
        "every 5 seconds",
        |_ctx| {
            Box::pin(async {
                println!("hi from 5 seconds job");
            })
        },
        "example::hi",
    )
    .await;

    // 3
    let _ctx = dirtybase_cron::CronJob::register(
        "0/10 * * * * ? *",
        |_ctx| {
            Box::pin(async {
                println!("hi from 10 seconds job");
            })
        },
        "example::hi2",
    )
    .await;

    // 4. Wait for 60 seconds before completing
    tokio::time::sleep(Duration::from_secs(60)).await;
    println!("completed");
}
