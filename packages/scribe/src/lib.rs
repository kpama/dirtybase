mod aggregate;
mod domain_event;
mod repository;

use dirtybase_contract::db::types::ArcUlidField;

pub use aggregate::*;
pub use domain_event::*;
pub use repository::*;

pub type AggregateId = ArcUlidField;
