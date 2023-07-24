use dirtybase_db::dirtybase_db_types::{
    field_values::FieldValue,
    types::{ColumnAndValue, FromColumnAndValue, IntoColumnAndValue},
    ColumnAndValueBuilder,
};

use super::SYS_ADMIN_TABLE_USER_ID_FIELD;

#[derive(Debug, Clone)]
pub struct SysAdminEntity {
    pub core_user_id: Option<String>,
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

impl FromColumnAndValue for SysAdminEntity {
    fn from_column_value(column_and_value: ColumnAndValue) -> Self {
        Self {
            core_user_id: FieldValue::from_ref_option_into(
                column_and_value.get(SYS_ADMIN_TABLE_USER_ID_FIELD),
            ),
        }
    }
}

impl IntoColumnAndValue for SysAdminEntity {
    fn into_column_value(self) -> ColumnAndValue {
        ColumnAndValueBuilder::new()
            .try_to_insert(SYS_ADMIN_TABLE_USER_ID_FIELD, self.core_user_id)
            .build()
    }
}
