use dirtybase_contract::db::macros::DirtyTable;
use dirtybase_contract::db::types::UlidField;

#[derive(Debug, Clone, DirtyTable)]
#[dirty(table = "core_sys_admin")]
pub struct SysAdminEntity {
    pub core_user_id: UlidField,
}

impl Default for SysAdminEntity {
    fn default() -> Self {
        Self { core_user_id: None }
    }
}

impl SysAdminEntity {
    pub fn new() -> Self {
        Self::default()
    }
}
