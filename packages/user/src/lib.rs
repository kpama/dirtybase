mod dirtybase_entry;

use std::ops::{Deref, DerefMut};

use dirtybase_contract::{
    db::{
        TableEntityTrait,
        field_values::FieldValue,
        types::{ColumnAndValue, FromColumnAndValue, IntoColumnAndValue},
    },
    user::model::{User, UserTrait},
};
use dirtybase_db::types::ArcUuid7;
use dirtybase_db_macro::DirtyTable;
pub use dirtybase_entry::*;

#[derive(DirtyTable, Default)]
#[dirty(table = "users")]
pub struct UserEntity {
    #[dirty(flatten)]
    user: UserWrapper,
}

impl Deref for UserEntity {
    type Target = User;
    fn deref(&self) -> &Self::Target {
        &self.user
    }
}

impl DerefMut for UserEntity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.user
    }
}

#[derive(Default, Clone)]
struct UserWrapper(User);

impl Deref for UserWrapper {
    type Target = User;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UserWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromColumnAndValue for UserWrapper {
    fn from_column_value(column_and_value: dirtybase_contract::db::types::ColumnAndValue) -> Self {
        Self::default()
    }
}

impl IntoColumnAndValue for UserWrapper {
    fn into_column_value(&self) -> dirtybase_contract::db::types::ColumnAndValue {
        let mut cv = ColumnAndValue::new();
        cv.insert("id".to_string(), ArcUuid7::default().into());
        cv.insert("username".to_string(), "james brown".into());
        cv.insert("password".to_string(), "password".into());
        cv.insert("salt".to_string(), "salt".into());
        cv.insert("email".to_string(), "email".into());

        cv
    }
}

impl From<FieldValue> for UserWrapper {
    fn from(value: FieldValue) -> Self {
        UserWrapper::default()
    }
}

impl Into<FieldValue> for UserWrapper {
    fn into(self) -> FieldValue {
        FieldValue::NotSet
    }
}

// impl UserTrait for UserEntity {
//     fn id_ref(&self) -> &dirtybase_contract::db::types::ArcUuid7 {
//         self.user.id_ref()
//     }

//     fn set_id(&mut self, id: dirtybase_contract::db::types::ArcUuid7) {
//         self.user.set_id(id);
//     }
// }

impl From<User> for UserEntity {
    fn from(value: User) -> Self {
        Self {
            user: UserWrapper(value),
        }
    }
}

impl From<UserEntity> for User {
    fn from(value: UserEntity) -> Self {
        value.user.0
    }
}
