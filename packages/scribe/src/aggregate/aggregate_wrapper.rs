use crate::DispatchedDomainEvent;

use super::Aggregate;

#[async_trait::async_trait]
pub trait AggregateTrait {
    async fn apply(&mut self, event: DispatchedDomainEvent);
    fn aggregate(&self) -> &Aggregate;
}
