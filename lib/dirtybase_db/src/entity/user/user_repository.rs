#![allow(dead_code)]

use super::{UserEntity, USER_TABLE};
use crate::base::{
    manager::Manager,
    types::{FromColumnAndValue, IntoColumnAndValue},
};

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

    pub fn manager_mut(&mut self) -> &mut Manager {
        &mut self.manager
    }

    pub async fn find_on_by_internal_id(&mut self, id: u64) -> Result<UserEntity, anyhow::Error> {
        match self
            .manager_mut()
            .select_from_table(USER_TABLE, |q| {
                q.select("*");
                q.eq("internal_id", id);
            })
            .fetch_one_as_field_value()
            .await
        {
            Ok(user) => Ok(UserEntity::from_column_value(user)),
            Err(e) => Err(e),
        }
    }

    pub async fn find_on_by_id(&mut self, id: &str) -> Result<UserEntity, anyhow::Error> {
        match self
            .manager_mut()
            .select_from_table(USER_TABLE, |q| {
                q.select("*");
                q.eq("id", id);
            })
            .fetch_one_as_field_value()
            .await
        {
            Ok(user) => Ok(UserEntity::from_column_value(user)),
            Err(e) => Err(e),
        }
    }

    pub async fn find_one_by_username_and_email(
        &mut self,
        username: &str,
        email: &str,
    ) -> Result<UserEntity, anyhow::Error> {
        match self
            .manager_mut()
            .select_from_table(USER_TABLE, |q| {
                q.select("*");
                q.eq("username", username).eq("email", email);
            })
            .fetch_one_as_field_value()
            .await
        {
            Ok(field) => {
                let user = UserEntity::from_column_value(field);
                Ok(user)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn create(
        &mut self,
        record: impl IntoColumnAndValue,
    ) -> Result<UserEntity, anyhow::Error> {
        let kv = record.into_column_value();
        self.manager.insert_record(USER_TABLE, kv).await;

        let fake = UserEntity::default();
        Ok(fake)
    }

    // Update an existing User record
    pub async fn update(
        &mut self,
        id: &str,
        record: impl IntoColumnAndValue,
    ) -> Result<UserEntity, anyhow::Error> {
        let kv = record.into_column_value();
        self.manager
            .save_record(USER_TABLE, kv, move |q| {
                q.eq("id", id);
            })
            .await;
        self.find_on_by_id(&id).await
    }
}
