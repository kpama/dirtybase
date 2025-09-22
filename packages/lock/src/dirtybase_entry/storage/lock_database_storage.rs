use dirtybase_contract::{db_contract::types::InternalIdField, lock_contract::LockData};
use dirtybase_db_macro::DirtyTable;

#[derive(Debug, Default, Clone, DirtyTable)]
#[dirty(table = "locks", no_soft_delete, no_timestamp)]
pub struct LockDataWrapper {
    id: InternalIdField,
    #[dirty(flatten)]
    data: LockData,
}
