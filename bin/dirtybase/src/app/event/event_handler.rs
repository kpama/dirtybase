use async_trait::async_trait;
use dirtybase_db::base::types::ColumnAndValue;

pub type Event = Option<ColumnAndValue>;

/// The trait that all handler must implement
#[async_trait]
pub trait EventListener: Send + Sync {
    async fn handle(&self, name: &str, event: &Event);

    /// A unique ID that identifies this handler.
    /// Event handler can be tricky to debug. This feature helps when
    /// debugging
    fn handler_id(&self) -> &str;
}
