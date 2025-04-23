use std::time::Duration;

use dirtybase_contract::{config_contract::DirtyConfig, prelude::Context};
use dirtybase_cron::{CronJobManager, CronJobRegisterer, JobHandlerWrapper, config::CronConfig};
use tracing::Level;

#[tokio::main]
async fn main() {
    // for logging purposes
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .try_init()
        .expect("could not setup tracing");

    // 1. Setup the configuration using the default config template
    let base_config = DirtyConfig::new();
    let config = base_config
        .optional_file("./config_template/cron.toml", Some("DTY_CRON"))
        .build()
        .await
        .unwrap()
        .try_deserialize::<CronConfig>()
        .unwrap();

    // 3. register a job
    CronJobRegisterer::register("foo::job", |_reg| {
        JobHandlerWrapper::new(|_ctx| {
            Box::pin(async {
                println!(">>>>>>>> running foo::bar......");
            })
        })
    })
    .await;

    // 4. Run the enabled jobs
    let manager = CronJobManager::new();
    manager.run(config, Context::make_global().await).await;

    // 5. This line is added just for testing purposes
    //    Usually, your program keep running due to the fact that it is accepting connection
    //    or doing something similar.
    tokio::time::sleep(Duration::from_secs(60)).await;

    // 6. Or you could loop
    // loop {
    //     tokio::time::sleep(Duration::from_secs(10)).await;
    // }

    println!("program ended");
}
