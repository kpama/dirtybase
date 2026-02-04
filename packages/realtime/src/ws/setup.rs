use futures_util::{sink::SinkExt, stream::StreamExt};
use std::{net::SocketAddr, ops::ControlFlow};

use axum::{
    RequestPartsExt,
    body::Bytes,
    extract::{
        ConnectInfo, FromRequestParts, WebSocketUpgrade,
        ws::{CloseFrame, Message, Utf8Bytes, WebSocket},
    },
    http::StatusCode,
    response::Response,
};

use crate::ws::RealtimeWebSocket;

pub struct RealtimeWebSocketSetup {
    ws: WebSocketUpgrade,
    socket_addr: SocketAddr,
}

impl RealtimeWebSocketSetup {
    pub fn response(self, namespace: &str) -> Response {
        if namespace.is_empty() {
            panic!("websocket namespace cannot be an empty string");
        }

        let add = self.socket_addr;
        let ns = namespace.to_string();
        self.ws
            .on_upgrade(move |socket| handle_ws_upgrade(socket, add, ns))
    }
}

async fn handle_ws_upgrade(mut socket: WebSocket, addr: SocketAddr, namespace: String) {
    println!("connection has been upgraded. namespace: {}", &namespace);

    if socket
        .send(Message::Ping(Bytes::from_static(&[1, 2, 3])))
        .await
        .is_ok()
    {
        println!("Pinged {addr}...");
    } else {
        println!("Could not send ping {addr}!");
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }

    let (mut sender, mut receiver) = socket.split();

    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        while let Some(Ok(msg)) = receiver.next().await {
            cnt += 1;
            // print message and break if instructed to do so
            if process_message(&msg, &addr).is_break() {
                break;
            }
            _ = RealtimeWebSocket::on_message(&namespace, msg).await;
        }
        cnt
    });

    // Spawn a task that will push several messages to the client (does not matter what client does)
    let mut send_task = tokio::spawn(async move {
        let n_msg = 20;
        for i in 0..n_msg {
            // In case of any websocket error, we exit.
            if sender
                .send(Message::Text(format!("Server message {i} ...").into()))
                .await
                .is_err()
            {
                return i;
            }

            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }

        println!("Sending close to {addr}...");
        if let Err(e) = sender
            .send(Message::Close(Some(CloseFrame {
                code: axum::extract::ws::close_code::NORMAL,
                reason: Utf8Bytes::from_static("Goodbye"),
            })))
            .await
        {
            println!("Could not send Close due to {e}, probably it is ok?");
        }
        n_msg
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        rv_a = (&mut send_task) => {
            match rv_a {
                Ok(a) => println!("{a} messages sent to {addr}"),
                Err(a) => println!("Error sending messages {a:?}")
            }
            recv_task.abort();
        },
        rv_b = (&mut recv_task) => {
            match rv_b {
                Ok(b) => println!("Received {b} messages"),
                Err(b) => println!("Error receiving messages {b:?}")
            }
            send_task.abort();
        }
    }
}

fn process_message(msg: &Message, who: &SocketAddr) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            println!(">>> {who} sent str: {t:?}");
        }
        Message::Binary(d) => {
            println!(">>> {who} sent {} bytes: {d:?}", d.len());
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> {who} sent close with code {} and reason `{}`",
                    cf.code, cf.reason
                );
            } else {
                println!(">>> {who} somehow sent close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            println!(">>> {who} sent pong with {v:?}");
        }
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            println!(">>> {who} sent ping with {v:?}");
        }
    }
    ControlFlow::Continue(())
}

impl<S> FromRequestParts<S> for RealtimeWebSocketSetup
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut dirtybase_contract::prelude::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Ok(ws) = parts.extract::<WebSocketUpgrade>().await else {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, String::new()));
        };

        let Ok(ConnectInfo(socket_addr)) = parts.extract::<ConnectInfo<SocketAddr>>().await else {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, String::new()));
        };

        return Ok(Self { ws, socket_addr });
    }
}
