use super::{Event, EventDispatcherBuilder, EventListener};
use async_trait::async_trait;

pub(crate) fn register_listeners(mut builder: EventDispatcherBuilder) -> EventDispatcherBuilder {
    builder.listen("system_ready", TestHandler);

    builder
}

struct TestHandler;

#[async_trait]
impl EventListener for TestHandler {
    async fn handle(&self, name: &str, _event: &Event) {
        log::info!("handling event: {}!", name);
    }

    fn handler_id(&self) -> &str {
        "Test event handler"
    }
}
