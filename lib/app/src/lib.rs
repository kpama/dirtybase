use app::AppService;
use clap::Parser;

pub mod app;
pub mod cli;
pub mod dirtybase_entry;
pub mod http;
pub use axum;
pub use dirtybase_contract as contract;

/// Setup database application using configs in .env files
pub async fn setup() -> anyhow::Result<AppService> {
    let config = app::Config::default();
    setup_using(&config).await
}

/// Setup database application using custom configuration
/// A builder exist that assist in building the configuration instance
/// ```rust
/// # use app::app::ConfigBuilder;
/// let builder = ConfigBuilder::new();
/// let config = builder.app_name("My awesome application")
///                     .web_port(8709)
///                     .build();
/// ```
///
pub async fn setup_using(config: &app::Config) -> anyhow::Result<AppService> {
    let pool_manager = dirtybase_db::setup(config.dirty_config()).await;
    let cache_manager = dirtybase_cache::setup(config.dirty_config()).await;

    let app = app::App::new(config, pool_manager, cache_manager).await?;
    app.register(dirtybase_entry::Extension).await;

    Ok(app)
}

pub async fn run_http(app_service: AppService) -> anyhow::Result<()> {
    log::info!("running web server");

    if app_service.config().web_enable_admin_routes()
        || app_service.config().web_enable_api_routes()
        || app_service.config().web_enable_general_routes()
    {
        http::init(app_service).await
    } else {
        Err(anyhow::anyhow!("No routes to register"))
    }
}

pub async fn run_cli(
    app_service: AppService,
    command: &app::command::Commands,
) -> anyhow::Result<()> {
    cli::init(app_service, command).await
}

pub async fn run(app_service: AppService) -> anyhow::Result<()> {
    let args = app::command::Args::parse();
    match &args.command {
        Some(app::command::Commands::Serve) => run_http(app_service.clone()).await,
        Some(command) => run_cli(app_service.clone(), command).await,
        None => Err(anyhow::anyhow!("Command was not handled")),
    }
}

pub async fn setup_and_run() -> anyhow::Result<()> {
    match setup().await {
        Ok(service) => run(service).await,
        Err(e) => Err(e),
    }
}

pub(crate) async fn shutdown_signal() {
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
            let app_service: AppService = busybody::helpers::provide().await;
            app_service.shutdown().await;
        },
        _ = terminate => {
            let app_service: AppService = busybody::helpers::provide().await;
            app_service.shutdown().await;
        },
    }
}
