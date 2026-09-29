use crate::{context::ActionContext, error::Result, query::Query};

/// A modification applied to a query before it reaches the data layer.
///
/// Preparations are the read side of the same idea as changes, and correspond to
/// Ash's `prepare build(...)`.
#[async_trait::async_trait]
pub trait Preparation: Send + Sync {
    async fn prepare(&self, query: &mut Query, context: &dyn ActionContext) -> Result<()>;

    fn describe(&self) -> Option<String> {
        None
    }
}

/// Lets a boxed preparation be handed to `Action::prepare` directly.
#[async_trait::async_trait]
impl Preparation for Box<dyn Preparation> {
    async fn prepare(&self, query: &mut Query, context: &dyn ActionContext) -> Result<()> {
        (**self).prepare(query, context).await
    }

    fn describe(&self) -> Option<String> {
        (**self).describe()
    }
}
