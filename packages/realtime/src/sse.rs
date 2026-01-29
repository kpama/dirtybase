use axum::body::Body;
use axum::http::Response;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::sse::Sse;
use dirtybase_contract::prelude::AppCancellationToken;
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::RwLock;
use std::time::Duration;
use tokio::sync::broadcast::error::SendError;
use tokio_util::sync::CancellationToken;

pub use axum::response::sse::Event;
pub use axum::response::sse::EventDataWriter;

#[derive(Clone)]
pub struct ServerSendEvent {
    cancel_token: CancellationToken,
    broadcaster: SseBroadcaster,
}

static BROADCASTERS: OnceLock<RwLock<HashMap<String, SseBroadcaster>>> = OnceLock::new();

#[derive(Clone)]
struct SseBroadcaster {
    sender: tokio::sync::broadcast::Sender<Event>,
    cancel_token: CancellationToken,
}

impl ServerSendEvent {
    /// Create a new or get back an existing instance with the specified namespace
    pub async fn new(ns: &str) -> Self {
        if ns.is_empty() {
            panic!("SSE namespace cannot be empty")
        }

        let cancel_token = busybody::helpers::get_type::<AppCancellationToken>()
            .await
            .expect("could not get app cancel token");
        let broadcasters = BROADCASTERS.get_or_init(RwLock::default);

        let mut lock = broadcasters
            .write()
            .expect("could not acquire sse broadcasters lock");
        let broadcaster = if let Some(b) = lock.get(ns).cloned() {
            b
        } else {
            let (tx, _) = tokio::sync::broadcast::channel::<Event>(100);
            //
            let b = SseBroadcaster {
                sender: tx.clone(),
                cancel_token: CancellationToken::new(),
            };
            lock.insert(ns.to_string(), b.clone());
            b
        };

        Self {
            broadcaster,
            cancel_token: cancel_token.into_inner(),
        }
    }

    /// Broadcast a new event to the clients
    pub fn send(&self, event: Event) -> Result<usize, SendError<Event>> {
        self.broadcaster.sender.send(event)
    }

    /// Close the stream
    /// Note: Client will receive a 204 status code after calling this method
    pub fn close(self) {
        self.broadcaster.cancel_token.cancel();
    }

    /// Checks if the application or this stream has been cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled() || self.broadcaster.cancel_token.is_cancelled()
    }

    /// Tranform this instance to an HTTP Response
    pub fn to_response(self) -> Response<Body> {
        let app_cancel_token = self.cancel_token.clone();
        let cancel_token = self.broadcaster.cancel_token.clone();

        let stream =
            tokio_stream::wrappers::BroadcastStream::new(self.broadcaster.sender.subscribe());

        if cancel_token.is_cancelled() || cancel_token.is_cancelled() {
            let mut resp = Response::new(Body::empty());
            *resp.status_mut() = StatusCode::NO_CONTENT;

            return resp;
        }

        Sse::new(stream.take_until(async move {
            while !app_cancel_token.is_cancelled() && !cancel_token.is_cancelled() {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            tracing::debug!("SSE connection closed");
            true
        }))
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                // .interval(Duration::from_secs(1)) // TODO: Make this configurable ???
                .text("keep-alive-text"),
        )
        .into_response()
    }
}

impl IntoResponse for ServerSendEvent {
    fn into_response(self) -> dirtybase_contract::prelude::Response {
        self.to_response()
    }
}
