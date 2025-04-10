use std::time::Duration;

use dirtybase_contract::config_contract::DirtyConfig;
use dirtybase_cron::config::CronConfig;
#[tokio::main]
async fn main() {
    // 1. Setup the configuration using the default config template
    let base_config = DirtyConfig::new();
    let config = base_config
        .optional_file("./config_template/cron.toml", Some("DTY_CRON"))
        .build()
        .await
        .unwrap()
        .try_deserialize::<CronConfig>()
        .unwrap();

    // 2. Setup cron manager
    let mut manager = dirtybase_cron::setup_using(config).await;

    // 3. register a job
    manager.register("foo::job", |_ctx| {
        Box::pin(async {
            println!(">>>>>>>> running foo::bar......");
        })
    });

    // 4. Run the enabled jobs
    manager.run().await;

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
