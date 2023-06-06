use super::event_handler::{Event, EventListener};
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedReceiver;

pub type DispatchedEvent = (String, Event);
pub type Subscribers = HashMap<String, Vec<Box<dyn EventListener>>>;

const LOG_TARGET: &str = "dirty:app:event-receiver";

// Receive and calls the actual handlers
pub(crate) struct EventReceiver {
    subscribers: Subscribers,
    chan_rev: UnboundedReceiver<DispatchedEvent>,
}

impl EventReceiver {
    pub fn new(subscribers: Subscribers, receiver: UnboundedReceiver<DispatchedEvent>) -> Self {
        Self {
            subscribers,
            chan_rev: receiver,
        }
    }

    pub async fn receive(&mut self) {
        while let Some((name, event)) = self.chan_rev.recv().await {
            if let Some(handlers) = self.subscribers.get(&name) {
                for a_handler in handlers {
                    a_handler.handle(&name, &event).await;
                    log::debug!(
                        target: LOG_TARGET,
                        "event: \"{}\"  handled by:  {:?}",
                        &name,
                        a_handler.handler_id()
                    );
                }
            }
        }
    }
}
