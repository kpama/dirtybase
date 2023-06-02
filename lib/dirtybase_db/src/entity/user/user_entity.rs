use crate::base::{
    field_values::FieldValue,
    helper::generate_ulid,
    types::{ColumnAndValue, FromColumnAndValue, IntoColumnAndValue},
    ColumnAndValueBuilder,
};

#[derive(Debug)]
pub struct UserEntity {
    internal_id: Option<u64>,
    id: String,
    username: String,
    email: String,
    reset_password: bool,
    password: String,
}

impl Default for UserEntity {
    fn default() -> Self {
        Self {
            internal_id: None,
            id: generate_ulid(),
            username: "".into(),
            email: "".into(),
            reset_password: false,
            password: "".into(), // MUST be the hashed of the raw password
        }
    }
}

impl FromColumnAndValue for UserEntity {
    fn from_column_value(column_and_value: ColumnAndValue) -> Self {
        Self {
            internal_id: if let Some(v) = column_and_value.get("internal_id") {
                Some(v.into())
            } else {
                None
            },
            id: FieldValue::pluck_some_or_default_ref(column_and_value.get("id")).into(),
            username: FieldValue::pluck_some_or_default_ref(column_and_value.get("username"))
                .into(),
            email: FieldValue::pluck_some_or_default_ref(column_and_value.get("email")).into(),
            reset_password: FieldValue::pluck_some_or_default_ref(
                column_and_value.get("reset_password"),
            )
            .into(),
            password: FieldValue::pluck_some_or_default_ref(column_and_value.get("password"))
                .into(),
        }
    }
}

impl UserEntity {
    pub fn internal_id(&self) -> Option<u64> {
        self.internal_id
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn set_id(&mut self, id: &str) -> &mut Self {
        self.id = id.into();
        self
    }

    pub fn username(&self) -> &String {
        &self.username
    }

    pub fn set_username(&mut self, username: &str) -> &mut Self {
        self.username = username.into();
        self
    }

    pub fn email(&self) -> &String {
        &self.email
    }

    pub fn set_email(&mut self, email: &str) -> &mut Self {
        self.email = email.into();
        self
    }

    pub fn password(&self) -> &String {
        &self.password
    }

    pub fn reset_password(&self) -> bool {
        self.reset_password
    }

    pub fn set_password(&mut self, password: &str) -> &mut Self {
        self.password = password.into();
        self
    }

    pub fn set_reset_password(&mut self, reset: bool) -> &mut Self {
        self.reset_password = reset;
        self
    }
}

pub struct UserUpdateEntity {
    pub id: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub reset_password: Option<bool>,
    pub password: Option<String>,
}

impl Default for UserUpdateEntity {
    fn default() -> Self {
        Self {
            id: None,
            username: None,
            email: None,
            reset_password: None,
            password: None,
        }
    }
}

impl IntoColumnAndValue for UserUpdateEntity {
    fn into_column_value(self) -> crate::base::types::ColumnAndValue {
        let builder = ColumnAndValueBuilder::new();

        if let Some(value) = self.id {
            builder.insert("id", value);
        }

        if let Some(value) = self.username {
            builder.insert("username", value);
        }

        if let Some(value) = self.email {
            builder.insert("email", value);
        }

        if let Some(value) = self.reset_password {
            builder.insert("reset_password", value);
        }

        if let Some(value) = self.password {
            builder.insert("password", value);
        }

        builder.build()
    }
}

impl From<UserEntity> for UserUpdateEntity {
    fn from(value: UserEntity) -> Self {
        Self {
            id: Some(value.id),
            username: Some(value.username),
            email: Some(value.email),
            reset_password: Some(value.reset_password),
            password: if value.password.is_empty() {
                None
            } else {
                Some(value.password)
            },
        }
    }
}
