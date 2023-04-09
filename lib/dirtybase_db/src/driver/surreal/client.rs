use futures_channel::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;

use super::payload::Payload;

pub struct Client {
    sender: UnboundedSender<Payload>,
}

impl Client {
    pub fn new(sender: UnboundedSender<Payload>) -> Self {
        Self { sender }
    }

    pub async fn query(&self, query: &str) -> Option<serde_json::Value> {
        let (receiver, payload) = Payload::new(Message::text(query));
        self.sender.unbounded_send(payload).unwrap();

        if let Some(data) = receiver.await.unwrap() {
            match serde_json::from_slice::<serde_json::Value>(&data) {
                Ok(value) => Some(value),
                Err(e) => {
                    println!("error: {:?}", e.to_string());
                    None
                }
            }
        } else {
            None
        }
    }
}
