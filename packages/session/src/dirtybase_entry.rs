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
        cookie: &CookieJar,
    ) -> Request {
        if let Ok(config) = context.get_config::<SessionConfig>("session").await {
            if let Ok(provider) = context.get::<SessionStorageProvider>().await {
                let request_session_id = cookie.get(config.cookie_id_ref());

                let id = match request_session_id {
                    Some(c) => {
                        // check the cookie
                        SessionId::from_str(c.value()).unwrap_or_default()
                    }
                    None => SessionId::new(),
                };

                if let Ok(h_context) = context.get::<HttpContext>().await {
                    let session =
                        Session::init(id, provider, config.lifetime(), &h_context.fingerprint())
                            .await;

                    tracing::trace!("adding session {} to request", session.id().to_string());

                    context.set(session).await;
                    context.set(config).await;
                }
            }
        } else {
            tracing::error!("could not setup request session")
        }

        req
    }

    async fn add_session_id_to_cookie(
        &self,
        resp: Response,
        mut cookie: CookieJar,
        context: Context,
    ) -> (Response, CookieJar) {
        if let Ok(session) = context.get::<Session>().await {
            if let Ok(config) = context.get::<SessionConfig>().await {
                println!(
                    "context instance still exist: session Id: {:?}",
                    session.id()
                );
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
