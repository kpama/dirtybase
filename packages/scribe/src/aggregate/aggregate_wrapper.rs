use crate::DispatchedDomainEvent;

use super::Aggregate;

pub trait AggregateTrait {
    fn apply(&mut self, event: DispatchedDomainEvent);
    fn aggregate(&self) -> &Aggregate;
}
