#![allow(dead_code)]

use super::{
    UserEntity, USER_TABLE, USER_TABLE_DELETED_AT_FIELD, USER_TABLE_EMAIL_FIELD,
    USER_TABLE_ID_FIELD, USER_TABLE_INTERNAL_ID_FIELD, USER_TABLE_USERNAME_FIELD,
};
use crate::base::manager::Manager;
use dirtybase_db_types::{field_values::FieldValue, types::IntoColumnAndValue};

pub struct UserRepository {
    manager: Manager,
    no_trashed: bool,
}

impl UserRepository {
    pub fn new(manager: Manager) -> Self {
        Self {
            manager,
            no_trashed: true,
        }
    }

    pub fn manager(&self) -> &Manager {
        &self.manager
    }

    /// Included soft deleted records
    pub fn with_trash(&mut self) -> &mut Self {
        self.no_trashed = true;
        self
    }

    /// Do not include soft deleted records
    pub fn without_trash(&mut self) -> &mut Self {
        self.no_trashed = false;
        self
    }

    pub async fn find_on_by_internal_id(&mut self, id: u64) -> Result<UserEntity, anyhow::Error> {
        self.manager()
            .select_from_table(USER_TABLE, |q| {
                q.select_all()
                    .eq(USER_TABLE_INTERNAL_ID_FIELD, id)
                    .and_is_null(USER_TABLE_DELETED_AT_FIELD);
            })
            .fetch_one_to()
            .await
    }

    pub async fn find_on_by_id(&self, id: &str) -> Result<UserEntity, anyhow::Error> {
        self.manager()
            .select_from_table(USER_TABLE, |q| {
                q.select_all()
                    .eq(USER_TABLE_ID_FIELD, id)
                    .and_is_null(USER_TABLE_DELETED_AT_FIELD);
            })
            .fetch_one_to()
            .await
    }

    pub async fn find_by_username_or_email(
        &self,
        username: &str,
        email: &str,
        without_trash: bool,
    ) -> Result<UserEntity, anyhow::Error> {
        if !username.is_empty() || !email.is_empty() {
            self.manager()
                .select_from_table(USER_TABLE, |q| {
                    q.select_all();
                    if without_trash {
                        q.without_trash();
                    }
                    if !email.is_empty() {
                        q.eq(USER_TABLE_EMAIL_FIELD, email);
                    } else {
                        q.eq(USER_TABLE_USERNAME_FIELD, username);
                    }
                })
                .fetch_one_to()
                .await
        } else {
            Err(anyhow::anyhow!("Both username and email values are empty"))
        }
    }

    pub async fn find_by_username_and_email(
        &self,
        username: &str,
        email: &str,
    ) -> Result<UserEntity, anyhow::Error> {
        self.manager()
            .select_from_table(USER_TABLE, |q| {
                q.select_all()
                    .eq(USER_TABLE_USERNAME_FIELD, username)
                    .eq(USER_TABLE_EMAIL_FIELD, email)
                    .is_null(USER_TABLE_DELETED_AT_FIELD);
            })
            .fetch_one_to()
            .await
    }

    pub async fn create(
        &self,
        record: impl IntoColumnAndValue,
    ) -> Result<UserEntity, anyhow::Error> {
        let column_and_values = record.into_column_value();
        let id: String =
            FieldValue::from_ref_option_into(column_and_values.get(USER_TABLE_ID_FIELD));
        self.manager().insert(USER_TABLE, column_and_values).await;

        self.find_on_by_id(&id).await
    }

    // Update an existing User record
    pub async fn update(
        &self,
        id: &str,
        record: impl IntoColumnAndValue,
    ) -> Result<UserEntity, anyhow::Error> {
        let kv = record.into_column_value();
        self.manager()
            .update(USER_TABLE, kv, move |q| {
                q.eq(USER_TABLE_ID_FIELD, id);
            })
            .await;
        self.find_on_by_id(&id).await
    }
}

// #[busybody::async_trait]
// impl busybody::Injectable for UserRepository {
//     async fn inject(c: &busybody::ServiceContainer) -> Self {
//         Self::new(c.proxy_value().unwrap())
//     }
// }
