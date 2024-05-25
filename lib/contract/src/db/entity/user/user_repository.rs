#![allow(dead_code)]

use super::UserEntity;
use crate::db::base::manager::Manager;
use dirtybase_db::{field_values::FieldValue, types::IntoColumnAndValue, TableEntityTrait};

pub struct UserRepository {
    manager: Manager,
}

impl UserRepository {
    pub fn new(manager: Manager) -> Self {
        Self { manager }
    }

    pub fn manager(&self) -> &Manager {
        &self.manager
    }

    pub async fn find_on_by_internal_id(
        &mut self,
        id: u64,
        without_trash: bool,
    ) -> Result<Option<UserEntity>, anyhow::Error> {
        self.manager()
            .select_from_table(UserEntity::table_name(), |q| {
                q.select_all()
                    .eq(UserEntity::col_name_for_internal_id(), id)
                    .and_is_null(UserEntity::col_name_for_deleted_at());
                if without_trash {
                    q.without_table_trash::<UserEntity>();
                }
            })
            .fetch_one_to()
            .await
    }

    pub async fn find_on_by_id(&self, id: &str) -> Result<Option<UserEntity>, anyhow::Error> {
        self.manager()
            .select_from_table(UserEntity::table_name(), |q| {
                q.select_all()
                    .eq(UserEntity::col_name_for_id(), id)
                    .and_is_null(UserEntity::col_name_for_deleted_at());
            })
            .fetch_one_to()
            .await
    }

    pub async fn find_by_username_or_email(
        &self,
        username: &str,
        email: &str,
        without_trash: bool,
    ) -> Result<Option<UserEntity>, anyhow::Error> {
        if !username.is_empty() || !email.is_empty() {
            self.manager()
                .select_from_table(UserEntity::table_name(), |q| {
                    q.select_all();
                    if without_trash {
                        q.without_table_trash::<UserEntity>();
                    }
                    if !email.is_empty() {
                        q.eq(UserEntity::col_name_for_email(), email);
                    } else {
                        q.eq(UserEntity::col_name_for_username(), username);
                    }
                })
                .fetch_one_to()
                .await
        } else {
            Err(anyhow::anyhow!("Both username and email values are empty"))
        }
    }

    pub async fn find_by_username(
        &self,
        username: &str,
        without_trash: bool,
    ) -> Result<Option<UserEntity>, anyhow::Error> {
        self.manager
            .select_from_table(UserEntity::table_name(), |query| {
                query
                    .select_all()
                    .eq(UserEntity::col_name_for_username(), username);

                if without_trash {
                    query.without_table_trash::<UserEntity>();
                }
            })
            .fetch_one_to()
            .await
    }

    pub async fn find_by_username_and_email(
        &self,
        username: &str,
        email: &str,
        without_trash: bool,
    ) -> Result<Option<UserEntity>, anyhow::Error> {
        self.manager()
            .select_from_table(UserEntity::table_name(), |q| {
                q.select_all()
                    .eq(UserEntity::col_name_for_username(), username)
                    .eq(UserEntity::col_name_for_email(), email);
                if without_trash {
                    q.without_table_trash::<UserEntity>();
                }
            })
            .fetch_one_to()
            .await
    }

    pub async fn create(
        &self,
        record: impl IntoColumnAndValue,
    ) -> Result<Option<UserEntity>, anyhow::Error> {
        let column_and_values = record.into_column_value();
        let id: String = FieldValue::from_ref_option_into(
            column_and_values.get(UserEntity::id_column().unwrap()),
        );
        self.manager()
            .insert(UserEntity::table_name(), column_and_values)
            .await;

        self.find_on_by_id(&id).await
    }

    // Update an existing User record
    pub async fn update(
        &self,
        id: &str,
        record: impl IntoColumnAndValue,
    ) -> Result<Option<UserEntity>, anyhow::Error> {
        let kv = record.into_column_value();
        self.manager()
            .update(UserEntity::table_name(), kv, move |q| {
                q.eq(UserEntity::id_column().unwrap(), id);
            })
            .await;
        self.find_on_by_id(id).await
    }
}

// #[busybody::async_trait]
// impl busybody::Injectable for UserRepository {
//     async fn inject(c: &busybody::ServiceContainer) -> Self {
//         Self::new(c.proxy_value().unwrap())
//     }
// }
