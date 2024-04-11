use dirtybase_contract::db::macros::DirtyTable;
use dirtybase_contract::db::types::UlidField;

#[derive(Debug, Clone, DirtyTable)]
#[dirty(table = "core_sys_admin")]
#[derive(Default)]
pub struct SysAdminEntity {
    pub core_user_id: UlidField,
}



impl SysAdminEntity {
    pub fn new() -> Self {
        Self::default()
    }
}
