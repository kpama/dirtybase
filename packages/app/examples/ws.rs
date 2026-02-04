use dirtybase_realtime::ws::{RealtimeWebSocket, RealtimeWebSocketSetup};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::{net::SocketAddr, ops::ControlFlow};

use axum::{
    Extension,
    body::Bytes,
    extract::{
        ConnectInfo, WebSocketUpgrade,
        ws::{CloseFrame, Message, Utf8Bytes, WebSocket},
    },
    response::{Html, IntoResponse},
};
use dirtybase_contract::{
    ExtensionSetup,
    http_contract::HttpContext,
    prelude::{CtxExt, RouterManager},
};

#[tokio::main]
async fn main() {
    let app = dirtybase_app::setup()
        .await
        .expect("could not create new application instance");

    // app.setup_web(|mut route_manager, _| {
    //     route_manager.general(None, |router| {
    //         router.get_x("/foo", || async { Html("<h1>Websocket example</h1") });
    //     });
    //     route_manager
    // })
    // .await;

    RealtimeWebSocket::handle_message("admin/chat-app", move |msg| {
        let msg = msg.clone();
        async move {
            println!("1st handling message: {:?}", msg.into_text());
        }
    })
    .await;

    RealtimeWebSocket::handle_message("admin/chat-app", move |msg| {
        if let Message::Text(txt) = msg {
            println!("2nd handling message: {:}", txt.as_str());
        }

        async {}
    })
    .await;

    app.register(WsApp).await;

    dirtybase_app::run(app)
        .await
        .expect("could not run application");
}

struct WsApp;

#[async_trait::async_trait]
impl ExtensionSetup for WsApp {
    fn register_routes(&self, manager: &mut RouterManager) {
        manager.general(None, |router| {
            router
                .get_x("/", index_page_handler)
                .get_x("/ws", |ws_setup: RealtimeWebSocketSetup| async move {
                    ws_setup.response("admin/chat-app")
                });
        });
    }
}

async fn index_page_handler() -> impl IntoResponse {
    let content = "<body>
  <h1>WebSocket Example</h1>

  <script>
    const ws = new WebSocket('/ws');
    ws.onopen = () => {
      console.log('connection established');
      setInterval(() => {
        if(ws.readyState != ws.CLOSED) {
            ws.send('client current timestamp: ' + Date.now());
        }
      }, 1000);
    }

    ws.onclose = () => {
      console.log('disconnected');
    }

    ws.onmessage = (msg) => {
      console.log('message', msg)
    } 
  </script>
</body>";

    Html(content)
}
