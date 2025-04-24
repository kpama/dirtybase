use core::AppService;

pub mod core;
pub mod dirtybase_entry;
pub mod http;

pub use async_trait;
pub use axum;
pub use busstop;
pub use busybody;
pub use clap;
pub use dirtybase_contract as contract;
use dirtybase_contract::cli_contract::setup_cli_command_manager;
pub use dirtybase_contract::config_contract;
pub use dirtybase_db as db;
pub use dirtybase_db_macro as db_macro;
pub use dirtybase_helper as helper;
pub use dirtybase_mail as mail;
pub use orsomafo;

/// Setup database application using configs in .env files
pub async fn setup() -> anyhow::Result<AppService> {
    let config = core::Config::new(None).await;
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
    busybody::helpers::set_type(config.dirty_config().clone()).await;

    let app = core::App::new(config).await?;

    // core extensions
    app.register(dirtybase_session::Extension).await;
    app.register(dirtybase_auth::Extension::default()).await;
    app.register(dirtybase_db::Extension).await;
    app.register(dirtybase_encrypt::Extension).await;
    // the core app
    app.register(dirtybase_entry::Extension).await;
    app.register(dirtybase_cache::Extension).await;
    app.register(dirtybase_cron::Extension).await;
    #[cfg(feature = "multitenant")]
    app.register(dirtybase_multitenant::Extension).await;
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

pub async fn run(app_service: AppService) -> anyhow::Result<()> {
    app_service.init().await;
    setup_cli_command_manager().await.handle().await;
    Ok(())
}

pub async fn setup_and_run() -> anyhow::Result<()> {
    match setup().await {
        Ok(app_service) => run(app_service).await,
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
            let app_service: AppService = busybody::helpers::get_type().await.expect("could not get app service");
            app_service.shutdown().await;
        },
        _ = terminate => {
            let app_service: AppService = busybody::helpers::get_type().await.expect("could not get app service");
            app_service.shutdown().await;
        },
    }
}
