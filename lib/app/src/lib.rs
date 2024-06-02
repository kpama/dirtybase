use core::AppService;

pub mod cli;
pub mod core;
pub mod dirtybase_entry;
pub mod http;
pub use axum;
pub use busybody;
pub use dirtybase_config as config;
pub use dirtybase_contract as contract;
pub use dirtybase_db as db;
pub use dirtybase_helper as helper;
pub use dirtybase_mail as mail;
pub use orsomafo;

use contract::cli::CliCommandManager;

/// Setup database application using configs in .env files
pub async fn setup() -> anyhow::Result<AppService> {
    let config = core::Config::default();
    setup_using(&config).await
}

/// Setup database application using custom configuration
/// A builder exist that assist in building the configuration instance
/// ```rust
/// # use core::core::ConfigBuilder;
/// let builder = ConfigBuilder::new();
/// let config = builder.app_name("My awesome application")
///                     .web_port(8709)
///                     .build();
/// ```
///
pub async fn setup_using(config: &core::Config) -> anyhow::Result<AppService> {
    let pool_manager = dirtybase_db::setup(config.dirty_config()).await;
    let cache_manager = dirtybase_cache::setup(config.dirty_config()).await;

    let app = core::App::new(config, pool_manager, cache_manager).await?;
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
    command: &core::command::Commands,
) -> anyhow::Result<()> {
    cli::init(app_service, command).await
}

pub async fn run(app_service: AppService) -> anyhow::Result<()> {
    let mut manager = CliCommandManager::new();

    let lock = app_service.extensions.read().await;
    for ext in lock.iter() {
        manager = ext.register_cli_commands(manager);
    }
    drop(lock);

    manager.handle(busybody::helpers::service_container()).await;
    Ok(())
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
