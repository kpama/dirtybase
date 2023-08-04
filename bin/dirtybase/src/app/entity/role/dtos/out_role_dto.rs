use dirtybase_db::macros::DirtyTable;

use crate::app::entity::role::RoleEntity;

#[derive(Debug, Default, Clone, serde::Serialize, DirtyTable)]
pub struct OutRoleEntityDto {
    pub id: String,
    name: String,
}

impl From<RoleEntity> for OutRoleEntityDto {
    fn from(value: RoleEntity) -> Self {
        Self {
            id: value.id.unwrap_or_default(),
            name: value.name.unwrap_or_default(),
        }
    }
}
