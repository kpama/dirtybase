use dirtybase_db::event::UserCreatedEvent;

mod user_created_handler;

pub fn register_event_handlers(
    builder: orsomafo::EventDispatcherBuilder,
) -> orsomafo::EventDispatcherBuilder {
    // event handlers
    builder.listen::<UserCreatedEvent, user_created_handler::UserCreatedHandler>()
    // .. more event handlers
}
