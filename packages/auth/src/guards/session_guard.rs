use std::net::SocketAddr;

use dirtybase_contract::{
    http::HttpContext,
    prelude::{
        ConnectInfo, FromRequest, FromRequestParts, HeaderValue,
        connect_info::Connected,
        header::{ACCESS_CONTROL_ALLOW_HEADERS, FORWARDED, HOST},
    },
    session::Session,
};

use crate::GuardResolver;

pub const SESSION_GUARD: &'static str = "session";

pub async fn authenticate(resolver: GuardResolver) -> GuardResolver {
    tracing::info!(">>>> In SESSION Authentication guard");
    if let Ok(session) = resolver.context_ref().get::<Session>().await {
        let user_id = session.get::<String>("user_id").await;
        let info = resolver
            .request_ref()
            .extensions()
            .get::<ConnectInfo<SocketAddr>>();

        if let Some(info) = info {
            tracing::warn!("connected client ip: {:#?}", info.ip());
        }

        // fetch IPs via proxies
        let trusted_proxies = ["forwarded", "x-forwarded-for"];
        for proxy in trusted_proxies {
            let value = resolver.request_ref().headers().get(proxy);
            tracing::warn!(">>> {}: {:?}", proxy, &value);
        }

        // hosts
        let host = resolver.request_ref().headers().get(HOST);
        tracing::warn!(">>>>> host: {:?}", host);

        tracing::warn!("current user id: {:?}", user_id);
        tracing::warn!("we go the session: {}", session.id());

        if let Ok(http_context) = resolver.context().get::<HttpContext>().await {
            //
            tracing::warn!(">>>> IP from http context: {:?}", http_context.ip())
        }
    }

    resolver
}
