use super::{
    event_handler::EventListener,
    event_receiver::{DispatchedEvent, EventReceiver, Subscribers},
};
use dirtybase_db::base::types::ColumnAndValue;
use std::fmt::Display;
use tokio::sync::mpsc::{self, UnboundedSender};

const LOG_TARGET: &str = "dirty:app:event-dispatcher";

#[derive(Debug, Clone)]
pub struct EventDispatcher {
    sender: UnboundedSender<DispatchedEvent>,
}

impl EventDispatcher {
    pub fn new(sender: UnboundedSender<DispatchedEvent>) -> Self {
        Self { sender }
    }

    /// Dispatches an event
    pub fn dispatch(&self, name: &str, event: ColumnAndValue) {
        self.do_dispatch(name, Some(event));
    }

    /// Dispatches an event with an empty/None payload
    pub fn whisper(&self, name: &str) {
        self.do_dispatch(name, None);
    }

    fn do_dispatch(&self, name: &str, event: Option<ColumnAndValue>) {
        log::debug!(target: LOG_TARGET, "dispatching event: {}", name);

        if let Err(e) = self.sender.send((name.to_owned(), event)) {
            log::error!(
                target: LOG_TARGET,
                "Could not send event: {}. error: {:?}",
                name,
                e.to_string()
            );
        }
    }
}

pub struct EventDispatcherBuilder {
    subscribers: Subscribers,
}

impl EventDispatcherBuilder {
    pub fn new() -> Self {
        Self {
            subscribers: Subscribers::new(),
        }
    }

    pub fn listen<E, H>(&mut self, name: E, handler: H) -> &mut Self
    where
        E: Display,
        H: EventListener + 'static,
    {
        let name_string: String = name.to_string();

        if self.subscribers.get(&name_string).is_none() {
            self.subscribers.insert(name_string.clone(), Vec::new());
        }

        if let Some(collection) = self.subscribers.get_mut(&name_string) {
            collection.push(Box::new(handler));
        }
        self
    }

    pub fn subscribe(&mut self, subscribers: Subscribers) -> &mut Self {
        for name_and_handlers in subscribers {
            if let Some(collection) = self.subscribers.get_mut(&name_and_handlers.0) {
                collection.extend(name_and_handlers.1);
            }
        }
        self
    }

    pub async fn build(self) -> EventDispatcher {
        let (tx, rx) = mpsc::unbounded_channel::<DispatchedEvent>();
        let subscribers = self.subscribers;

        tokio::spawn(async move {
            let mut handler = EventReceiver::new(subscribers, rx);
            handler.receive().await;
        });

        EventDispatcher::new(tx)
    }
}
