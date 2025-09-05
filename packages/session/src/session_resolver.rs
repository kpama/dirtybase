use dirtybase_contract::{
    http_contract::HttpContext,
    prelude::{
        Context,
        axum_extra::extract::{CookieJar, cookie::Cookie},
    },
    session_contract::{Session, SessionId, SessionStorageProvider},
};

use crate::SessionConfig;

#[derive(Clone)]
struct SessionAccessed(()); // Flag used to send session ID in cookie

pub(crate) async fn register_session_resolver(context: &Context) {
    context
        .container()
        .resolver::<Session>(move |ci| {
            Box::pin(async move {
                let context = ci
                    .get_type::<Context>()
                    .await
                    .expect("could not get context from CI");
                if let Ok(config) = context.get_config::<SessionConfig>("session").await
                    && let Ok(provider) = context.get::<SessionStorageProvider>().await
                {
                    let h_context = context.get::<HttpContext>().await.unwrap();
                    let cookie = h_context.cookie_jar().await;

                    let id = if let Some(c) = cookie.get(config.cookie_id_ref()) {
                        let id = SessionId::from_str_ref(c.value());
                        tracing::trace!("request session id: {}", id.as_ref().unwrap());
                        id
                    } else {
                        tracing::trace!("request does not have an active session");
                        None
                    };

                    let session = Session::init(id, provider, config.lifetime(), &context).await;

                    context.set(SessionAccessed(())).await;
                    return session;
                }
                tracing::error!("could not setup request session");
                panic!("could not setup session")
            })
        })
        .await;
}

pub(crate) async fn attach_session_cookie(context: &Context, mut cookie: CookieJar) -> CookieJar {
    if context.get::<SessionAccessed>().await.is_err() {
        return cookie;
    }

    if let Ok(session) = context.get::<Session>().await
        && let Ok(config) = context.get::<SessionConfig>().await
    {
        tracing::trace!("adding session {} to request", session.id());

        session.save().await;
        let mut entry = Cookie::new(config.cookie_id().to_string(), session.id().to_string());

        let mut ts = cookie::time::OffsetDateTime::now_utc();
        ts += cookie::time::Duration::minutes(config.lifetime() * 2);
        entry.set_expires(ts);
        entry.set_path("/");
        cookie = cookie.add(entry);
    }

    cookie
}
