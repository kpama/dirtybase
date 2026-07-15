use crate::DispatchedDomainEvent;

use super::Aggregate;

#[async_trait::async_trait]
pub trait AggregateTrait {
    async fn apply(self, event: DispatchedDomainEvent) -> Self;
    fn aggregate(&self) -> &Aggregate;
}
