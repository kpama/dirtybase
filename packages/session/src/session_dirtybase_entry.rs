use std::sync::Arc;

use dirtybase_contract::{
    app::Context,
    async_trait,
    axum::response::Response,
    config::DirtyConfig,
    prelude::{
        axum_extra::extract::{cookie::Cookie, CookieJar},
        Request,
    },
    session::{Session, SessionId, SessionStorage, SessionStorageProvider},
    ExtensionSetup,
};

use crate::{storage::MemoryStorage, SessionConfig, SessionStorageDriver};

#[derive(Default)]
pub struct Extension {
    config: SessionConfig,
}

#[async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, app_config: &DirtyConfig) {
        println!(">> session setup method called");
        // // TODO: Source the storage type for a config

        let config = SessionConfig::from(app_config);
        println!("{:?}", &config);

        self.setup_session_storage(&config).await;
        self.config = config;
    }

    async fn on_web_request(
        &self,
        mut req: Request,
        context: Context,
        cookie_jar: &CookieJar,
    ) -> Request {
        req = self.add_session_to_request(req, context, cookie_jar).await;

        req
    }

    async fn on_web_response(
        &self,
        resp: Response,
        cookie: CookieJar,
        context: Context,
    ) -> (Response, CookieJar) {
        let result = self.add_session_id_to_cookie(resp, cookie, context);

        result
    }
}

impl Extension {
    async fn add_session_to_request(
        &self,
        mut req: Request,
        context: Context,
        cookie: &CookieJar,
    ) -> Request {
        let request_session_id = cookie.get(self.config.cookie_ref());
        let session_storage_provider = context
            .container_ref()
            .get_type::<Arc<SessionStorageProvider>>()
            .unwrap();

        tracing::trace!("request has session id {:?}", request_session_id.is_some());

        let id = match request_session_id {
            Some(c) => {
                // check the cookie
                SessionId::from_str(&c.value().to_string()).unwrap_or_default()
            }
            None => SessionId::new(),
        };

        let session =
            Session::find_or_new(id, session_storage_provider, self.config.lifetime()).await;
        tracing::trace!("adding session {} to request", session.id().to_string());

        context.set(session.clone());
        req.extensions_mut().insert(session);

        req
    }

    fn add_session_id_to_cookie(
        &self,
        resp: Response,
        mut cookie: CookieJar,
        context: Context,
    ) -> (Response, CookieJar) {
        if let Some(session) = context.get::<Session>() {
            println!(
                "context instance still exist: session Id: {:?}",
                session.id()
            );
            cookie = cookie.add(Cookie::new(
                self.config.cookie().to_string(),
                session.id().to_string(),
            ));
        } else {
            println!(">>> could not get the current session");
        }

        (resp, cookie)
    }

    async fn setup_session_storage(&self, config: &SessionConfig) {
        println!("current session storage driver: {:?}", config.driver());
        match config.driver() {
            SessionStorageDriver::Memory => {
                log::debug!(
                    "current session storage provider: {:?}",
                    SessionStorageDriver::Memory
                );
                let provider = MemoryStorage::make_provider().await;

                // In memory cron job
                let lifetime = config.lifetime();
                let _ctx = dirtybase_cron::CronJob::register(
                    "every 5 minutes",
                    move |_| {
                        Box::pin({
                            let storage = provider.clone();
                            async move {
                                storage.gc(lifetime).await;
                            }
                        })
                    },
                    "session::memory-storage",
                )
                .await;
            }
            _ => todo!("session storage driver not implemented"),
        }
        println!("session config: {:?}", config);
    }
}
