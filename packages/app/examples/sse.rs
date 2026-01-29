use axum::{extract::Path, response::Html};
use dirtybase_contract::{ExtensionSetup, prelude::RouterManager};

use axum::response::sse::Event;
use dirtybase_helper::time::{now, now_ts};
use dirtybase_realtime::sse::ServerSendEvent;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let app = dirtybase_app::setup().await.expect("could not setup app");

    (SseApp).register().await;

    tokio::task::spawn(async move {
        let sse = ServerSendEvent::new("global").await;
        let mut counter = 0;

        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            let event = Event::default()
                .event("global::message")
                .data(format!("::GLOBAL MESSAGE:: counter: {}", counter))
                .id(format!("{}", now_ts()));
            _ = sse.send(event);

            counter += 1;
        }
    });

    tokio::task::spawn(async move {
        let sse = ServerSendEvent::new("global").await;
        loop {
            _ = sse.send(
                Event::default()
                    .event("server::time")
                    .data(now().as_datetime().format("%Y-%m-%d %H:%M:%S").to_string()),
            );
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    _ = dirtybase_app::run(app).await;
}

struct SseApp;

#[async_trait::async_trait]
impl ExtensionSetup for SseApp {
    fn register_routes(&self, manager: &mut RouterManager) {
        manager.general(None, |router| {
            router
                .get_x("/", || async move {
                    Html(
                        "<body> 
                            <h1>Hello world</h1>
                            <h2> Server Time: <span id='time'></span></h2>
                        </body>
                        <script>
                            const evtSource = new EventSource('/sse');
                            evtSource.onmessage = (msg) => console.log(msg);
                            evtSource.addEventListener('global::message', (event) => {
                                console.info('global event', event);
                            });
                            evtSource.addEventListener('server::time', (event) => {
                                document.getElementById('time').innerHTML = event.data;
                                // console.log('server time', event);
                            });
                    </script>",
                    )
                })
                .get_x("/scores", || async {
                    Html(
                        "<body> 
                            <h1>Scores</h1>
                            <h2><span id='score'></span></h2>
                        </body>
                        <script>
                            const evtSource = new EventSource('/sse-score');
                            // evtSource.onmessage = (msg) => console.log(msg);
                            // evtSource.addEventListener('global::message', (event) => {
                            //     console.info('global event', event);
                            // });
                            evtSource.addEventListener('game::score', (event) => {
                                document.getElementById('score').innerHTML = event.data;
                            });
                    </script>",
                    )
                })
                .get_x(
                    "/update-score/{score}",
                    |Path(score): Path<i32>| async move {
                        let event = Event::default()
                            .data(score.to_string())
                            .event("game::score");

                        if let Err(e) = ServerSendEvent::new("game-scores").await.send(event) {
                            eprintln!("error broadcasting score: {}", e);
                        }
                        "updated successfully"
                    },
                )
                .get_x("/sse", || async {
                    dirtybase_realtime::sse::ServerSendEvent::new("global").await
                })
                .get_x("/sse-score", || async {
                    ServerSendEvent::new("game-scores").await
                });
        });
    }
}
