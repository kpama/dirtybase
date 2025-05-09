mod migration;

use dirtybase_contract::{
    ExtensionMigrations, ExtensionSetup,
    app_contract::Context,
    async_trait,
    axum::response::Response,
    http_contract::HttpContext,
    prelude::{
        Request,
        axum_extra::extract::{CookieJar, cookie::Cookie},
    },
    session_contract::{Session, SessionId, SessionStorageProvider},
};

use crate::{SessionConfig, resource_manager::register_resource_manager};

#[derive(Default)]
pub struct Extension;

#[derive(Clone)]
struct SessionAccessed(()); // flag used to send session ID in cookie

#[async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, global_context: &Context) {
        if let Ok(config) = global_context.get_config::<SessionConfig>("session").await {
            global_context.set(config).await;
        }

        register_resource_manager().await;
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
        self.add_session_id_to_cookie(resp, cookie, context).await
    }

    fn migrations(&self, _context: &Context) -> Option<ExtensionMigrations> {
        migration::setup()
    }
}

impl Extension {
    async fn add_session_to_request(
        &self,
        req: Request,
        context: Context,
        _cookie: &CookieJar,
    ) -> Request {
        context
            .container()
            .resolver::<Session>(move |_| {
                let context = context.clone();
                Box::pin(async move {
                    if let Ok(config) = context.get_config::<SessionConfig>("session").await {
                        if let Ok(provider) = context.get::<SessionStorageProvider>().await {
                            let h_context = context.get::<HttpContext>().await.unwrap();
                            let cookie = h_context.cookie_jar().await;

                            let id = match cookie.get(config.cookie_id_ref()) {
                                Some(c) => SessionId::from_str(c.value()).unwrap_or_default(),
                                None => SessionId::new(),
                            };

                            let session = Session::init(
                                id,
                                provider,
                                config.lifetime(),
                                &h_context.fingerprint(),
                            )
                            .await;

                            tracing::trace!(
                                "adding session {} to request",
                                session.id().to_string()
                            );

                            context.set(SessionAccessed(())).await;
                            return session;
                        }
                    }
                    tracing::error!("could not setup request session");
                    panic!("could not setup session")
                })
            })
            .await;

        req
    }

    async fn add_session_id_to_cookie(
        &self,
        resp: Response,
        mut cookie: CookieJar,
        context: Context,
    ) -> (Response, CookieJar) {
        if context.get::<SessionAccessed>().await.is_err() {
            return (resp, cookie); // we don't need to set the session's ID cookie
        }

        if let Ok(session) = context.get::<Session>().await {
            if let Ok(config) = context.get::<SessionConfig>().await {
                tracing::trace!("session id: {}", session.id());
                let mut entry =
                    Cookie::new(config.cookie_id().to_string(), session.id().to_string());

                let mut ts = cookie::time::OffsetDateTime::now_utc();
                ts += cookie::time::Duration::minutes(config.lifetime());
                entry.set_expires(ts);
                entry.set_path("/");
                cookie = cookie.add(entry);
            }
        }

        (resp, cookie)
    }
}
