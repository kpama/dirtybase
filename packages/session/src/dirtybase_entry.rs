use std::sync::Arc;

use dirtybase_contract::{
    ExtensionSetup,
    app::Context,
    async_trait,
    axum::response::Response,
    prelude::{
        Request,
        axum_extra::extract::{CookieJar, cookie::Cookie},
    },
    session::{Session, SessionId, SessionStorageProvider},
};

use crate::{SessionConfig, resource_manager::register_resource_manager, storage::MemoryStorage};

#[derive(Default)]
pub struct Extension;

#[async_trait]
impl ExtensionSetup for Extension {
    async fn setup(&mut self, global_context: &Context) {
        println!(">> session setup method called");
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
        let result = self.add_session_id_to_cookie(resp, cookie, context).await;

        result
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
            if let Ok(provider) = context.get::<Arc<SessionStorageProvider>>().await {
                let request_session_id = cookie.get(config.cookie_id_ref());

                let id = match request_session_id {
                    Some(c) => {
                        // check the cookie
                        SessionId::from_str(&c.value().to_string()).unwrap_or_default()
                    }
                    None => SessionId::new(),
                };

                let session = Session::init(id, provider, config.lifetime()).await;
                tracing::trace!("adding session {} to request", session.id().to_string());

                context.set(session).await;
                context.set(config).await;
            }
        } else {
            tracing::error!("could not setup request sesssion")
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
                cookie = cookie.add(Cookie::new(
                    config.cookie_id().to_string(),
                    session.id().to_string(),
                ));
            }
        }

        (resp, cookie)
    }
}
