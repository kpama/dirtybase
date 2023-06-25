use super::{DIRTYBASE_USER_TABLE_CORE_USER_FIELD, DIRTYBASE_USER_TABLE_LOGIN_ATTEMP_FIELD};
use crate::app::entity::dirtybase_user::DIRTYBASE_USER_TABLE_LAST_LOGIN_FIELD;
use dirtybase_db::base::{
    field_values::FieldValue,
    types::{DateTimeField, FromColumnAndValue, IntoColumnAndValue, SingedInteger, UlidField},
    ColumnAndValueBuilder,
};

pub struct DirtybaseUserEntity {
    pub core_user_id: UlidField,
    pub login_attemp: SingedInteger,
    pub last_login_at: DateTimeField,
}

impl Default for DirtybaseUserEntity {
    fn default() -> Self {
        Self {
            core_user_id: None,
            login_attemp: None,
            last_login_at: None,
        }
    }
}

impl FromColumnAndValue for DirtybaseUserEntity {
    fn from_column_value(column_and_value: dirtybase_db::base::types::ColumnAndValue) -> Self {
        Self {
            core_user_id: FieldValue::from_ref_option_into(
                column_and_value.get(DIRTYBASE_USER_TABLE_CORE_USER_FIELD),
            ),
            login_attemp: FieldValue::from_ref_option_into(
                column_and_value.get(DIRTYBASE_USER_TABLE_LOGIN_ATTEMP_FIELD),
            ),
            last_login_at: FieldValue::from_ref_option_into(
                column_and_value.get(DIRTYBASE_USER_TABLE_LAST_LOGIN_FIELD),
            ),
        }
    }
}

impl IntoColumnAndValue for DirtybaseUserEntity {
    fn into_column_value(self) -> dirtybase_db::base::types::ColumnAndValue {
        ColumnAndValueBuilder::new()
            .try_to_insert(DIRTYBASE_USER_TABLE_CORE_USER_FIELD, self.core_user_id)
            .try_to_insert(DIRTYBASE_USER_TABLE_LOGIN_ATTEMP_FIELD, self.login_attemp)
            .try_to_insert(DIRTYBASE_USER_TABLE_LAST_LOGIN_FIELD, self.last_login_at)
            .build()
    }
}
