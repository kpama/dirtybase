use core::AppService;

pub mod cli;
pub mod core;
pub mod dirtybase_entry;
pub mod http;

pub use async_trait;
pub use axum;
pub use busstop;
pub use busybody;
pub use clap;
pub use dirtybase_contract as contract;
pub use dirtybase_contract::config;
use dirtybase_contract::{app::Context, cli::CliMiddlewareManager, ExtensionManager};
pub use dirtybase_db as db;
pub use dirtybase_db_macro as db_macro;
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
/// # use dirtybase_app::core::ConfigBuilder;
/// let builder = ConfigBuilder::new();
/// let config = builder.app_name("My awesome application")
///                     .web_port(8709)
///                     .build();
/// ```
///
pub async fn setup_using(config: &core::Config) -> anyhow::Result<AppService> {
    busybody::helpers::register_service(config.dirty_config().clone()).await;
    busybody::helpers::set_type(Context::make_global().await).await;

    let app = core::App::new(config).await?;

    // core extensions
    app.register(dirtybase_session::Extension::default()).await;
    app.register(dirtybase_auth::Extension::default()).await;
    app.register(dirtybase_db::Extension::default()).await;
    app.register(dirtybase_entry::Extension::default()).await;
    app.register(dirtybase_cache::Extension::default()).await;
    app.register(dirtybase_cron::Extension::default()).await;
    #[cfg(feature = "multitenant")]
    app.register(dirtybase_multitenant::Extension::default())
        .await;

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
    let manager = make_cli_command_manager(app_service).await;
    manager.handle().await;
    Ok(())
}

pub async fn run_command<I, T>(command: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<String>,
{
    let app_service = if let Some(s) = busybody::helpers::get_type::<AppService>().await {
        s
    } else {
        setup().await?
    };

    make_cli_command_manager(app_service)
        .await
        .handle_command(Some(command))
        .await;
    Ok(())
}

pub async fn run_command_with<I, T>(command: I, app_service: AppService) -> anyhow::Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<String>,
{
    make_cli_command_manager(app_service)
        .await
        .handle_command(Some(command))
        .await;
    Ok(())
}

pub async fn setup_and_run() -> anyhow::Result<()> {
    match setup().await {
        Ok(service) => run(service).await,
        Err(e) => Err(e),
    }
}

pub(crate) async fn make_cli_command_manager(app_service: AppService) -> CliCommandManager {
    app_service.init().await;

    let lock = ExtensionManager::list().read().await;

    let mut middleware = CliMiddlewareManager::new();
    for ext in lock.iter() {
        middleware = ext.register_cli_middlewares(middleware);
    }
    let mut manager = CliCommandManager::new(middleware);

    for ext in lock.iter() {
        manager = ext.register_cli_commands(manager);
    }
    drop(lock);
    manager
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
