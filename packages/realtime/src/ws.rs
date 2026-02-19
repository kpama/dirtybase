mod setup;
use std::sync::Arc;

use axum::extract::ws::Message;
use busybody::Service;
pub use setup::*;

pub struct RealtimeWebSocket;

impl RealtimeWebSocket {
    pub async fn on_message(namespace: &str, msg: Message) {
        Self::get_message_manager()
            .await
            .send((namespace.to_string(), msg))
            .await;
    }

    pub async fn handle_message<F, FR>(namespace: &str, callback: F)
    where
        F: Clone + Fn(&Message) -> FR + Send + Sync + 'static,
        FR: Future<Output = ()> + Send + 'static,
    {
        let manager = Self::get_message_manager().await;

        let ns_arc = Arc::new(namespace.to_string());
        manager
            .next(move |(ns, msg), next| {
                let cb = callback.clone();
                let n = ns_arc.clone();
                async move {
                    if *n == ns {
                        cb(&msg).await;
                        next.call((ns, msg)).await
                    }
                }
            })
            .await;
    }

    async fn get_message_manager() -> Service<simple_middleware::Manager<(String, Message), ()>> {
        if let Some(m) =
            busybody::helpers::get_service::<simple_middleware::Manager<(String, Message), ()>>()
                .await
        {
            m
        } else {
            let m = simple_middleware::Manager::<(String, Message), ()>::last(
                |(ns, _msg), _| async move {
                    println!(">> last middleware handler for {}", ns);
                },
            )
            .await;
            busybody::helpers::service_container()
                .set(m)
                .await
                .get()
                .await
                .unwrap() // NOTE: Will never fail
        }
    }
}
