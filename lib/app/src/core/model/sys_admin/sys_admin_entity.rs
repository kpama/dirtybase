use dirtybase_contract::db::types::UlidField;
use dirtybase_db_macro::DirtyTable;

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
