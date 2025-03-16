use dirtybase_contract::{ExtensionSetup, cli::CliCommandManager};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    let app_service = dirtybase_app::setup().await?;

    app_service.register(MyApp).await;

    dirtybase_app::run(app_service.clone()).await
}

struct MyApp;

#[async_trait::async_trait]
impl ExtensionSetup for MyApp {
    fn register_cli_commands(&self, mut manager: CliCommandManager) -> CliCommandManager {
        manager.register(
            clap::Command::new("hello")
                .about("greet the user with a `hello xxxx world`")
                .arg(clap::Arg::new("name").short('n').long("name")),
            |_name, mut matches, _service| {
                Box::pin(async move {
                    let name = matches
                        .remove_one("name")
                        .unwrap_or_else(|| "Unknown".to_string());
                    println!("Hello {} !!!", &name);
                    Ok(())
                })
            },
        );

        manager
    }
}
