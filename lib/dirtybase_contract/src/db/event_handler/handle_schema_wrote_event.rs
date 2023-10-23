use orsomafo::DispatchedEvent;

use crate::db::{event::SchemeWroteEvent, LAST_WRITE_TS};

#[derive(Debug, Default)]
pub struct HandleSchemaWroteEvent;

#[orsomafo::async_trait]
impl orsomafo::EventHandler for HandleSchemaWroteEvent {
    /// After a write has occurred for a particular kind of database
    /// We will try to log the timestamp. This value is used when the "sticky"
    /// feature is turned on. Please review the crate's config file
    async fn handle(&self, dispatched: &DispatchedEvent) {
        if let Some(event) = dispatched.the_event::<SchemeWroteEvent>() {
            if let Some(log) = LAST_WRITE_TS.get() {
                if let Ok(mut lock) = log.write() {
                    let ts = chrono::Utc::now().timestamp();
                    if !lock.contains_key(event.kind()) {
                        lock.insert(event.kind().clone(), ts);
                    } else if let Some(entry) = lock.get_mut(event.kind()) {
                        *entry = ts;
                    }
                    log::debug!("db last write occurred at: {:?}", ts);
                }
            }
        }
    }
}
