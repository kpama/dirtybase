use dirtybase_db::types::UlidField;
use dirtybase_db_macro::DirtyTable;

use crate::core::model::role::RoleEntity;

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize, DirtyTable)]
pub struct OutRoleEntityDto {
    pub id: UlidField,
    name: String,
}

impl From<RoleEntity> for OutRoleEntityDto {
    fn from(value: RoleEntity) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}
