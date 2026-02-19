use dirtybase_contract::{
    db_contract::types::{IntegerField, OptionalInternalIdField, StringField},
    lock_contract::LockData,
};
use dirtybase_db_macro::DirtyTable;

#[derive(Debug, Default, Clone, DirtyTable)]
#[dirty(table = "locks", no_soft_delete, no_timestamp)]
pub struct LockDataWrapper {
    id: OptionalInternalIdField,
    key: StringField,
    owner: StringField,
    expires: IntegerField,
}

impl From<LockData> for LockDataWrapper {
    fn from(data: LockData) -> Self {
        Self {
            id: None,
            key: data.key().to_string().into(),
            owner: data.owner().to_string().into(),
            expires: data.expires(),
            // data,
        }
    }
}
