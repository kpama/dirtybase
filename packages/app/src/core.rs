mod config;

use std::convert::Infallible;
use std::ops::Deref;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

pub use config::Config;
pub use config::ConfigBuilder;
pub use config::CookieConfig;

use dirtybase_contract::ExtensionManager;
use dirtybase_contract::app_contract::Context;
use dirtybase_contract::config_contract::DirtyConfig;
use dirtybase_contract::http_contract::RouterManager;
use dirtybase_contract::http_contract::WebMiddlewareManager;
use dirtybase_contract::prelude::AppCancellationToken;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

pub type AppService = busybody::Service<App>;

pub(crate) struct WebSetup(pub(crate) RouterManager, pub(crate) WebMiddlewareManager);

impl WebSetup {
    pub(crate) fn new(config: &Config) -> Self {
        let manager = RouterManager::new(
            config.web_api_route_prefix(),
            config.web_admin_route_prefix(),
            config.web_insecure_api_route_prefix(),
            config.web_dev_route_prefix(),
        );
        let middleware_manager = WebMiddlewareManager::new();
        Self(manager, middleware_manager)
    }
}

pub struct App {
    config: Config,
    pub(crate) web_setup: RwLock<Option<WebSetup>>,
    cancel_token: AppCancellationToken,
}

impl App {
    pub async fn new(config: &Config) -> anyhow::Result<AppService> {
        let instance = Self {
            config: config.clone(),
            web_setup: RwLock::default(),
            cancel_token: AppCancellationToken::new(CancellationToken::new()),
        };

        busybody::helpers::service_container()
            .set_type(instance.cancel_token.clone())
            .await;
        busybody::helpers::service_container().set(instance).await;

        Ok(busybody::helpers::service_container()
            .get::<Self>()
            .await
            .unwrap())
    }

    pub async fn register(
        &self,
        extension: impl dirtybase_contract::ExtensionSetup + 'static,
    ) -> &Self {
        ExtensionManager::register(extension).await;
        self
    }

    pub async fn global_context(&self) -> Context {
        dirtybase_contract::app_contract::global_context().await
    }
    pub async fn init(&self) {
        ExtensionManager::setup_boot_run(&self.global_context().await).await;
    }

    pub async fn shutdown(&self) {
        ExtensionManager::shutdown(&self.global_context().await).await;
        self.cancel_token.clone().into_inner().cancel();
    }

    pub fn cancel_token(&self) -> AppCancellationToken {
        self.cancel_token.clone()
    }

    pub async fn extensions(
        &self,
        callback: impl FnMut(&Box<dyn dirtybase_contract::ExtensionSetup>),
    ) {
        ExtensionManager::extensions(callback).await;
    }

    pub fn config(&self) -> Config {
        self.config.clone()
    }

    /// Setup a quick web application
    /// Instead of creating an extension, this method can be used
    /// to register web routers and middleware.
    pub async fn setup_web<F>(&self, mut callback: F)
    where
        F: FnMut(RouterManager, &mut WebMiddlewareManager) -> RouterManager,
    {
        if ExtensionManager::is_ready().await {
            return;
        }

        let mut w_lock = self.web_setup.write().await;
        let WebSetup(mut m, mut mm) = if let Some(web_setup) = w_lock.take() {
            web_setup
        } else {
            WebSetup::new(&self.config)
        };

        m = callback(m, &mut mm);

        w_lock.replace(WebSetup(m, mm));
    }

    pub fn dirty_config(&self) -> DirtyConfig {
        self.config.dirty_config().clone()
    }

    pub fn dirty_config_ref(&self) -> &DirtyConfig {
        self.config.dirty_config()
    }

    pub fn config_ref(&self) -> &Config {
        &self.config
    }
}

pub struct AppServiceExtractor(AppService);

impl AppServiceExtractor {
    pub fn inner(self) -> AppService {
        self.0
    }
}

impl Deref for AppServiceExtractor {
    type Target = AppService;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<AppServiceExtractor> for AppService {
    fn from(value: AppServiceExtractor) -> Self {
        value.0
    }
}

impl<S> FromRequestParts<S> for AppServiceExtractor
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(AppServiceExtractor(
            busybody::helpers::get_service().await.unwrap(),
        ))
    }
}
