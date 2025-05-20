#![allow(dead_code)]

use chrono::{DateTime, Utc};

use crate::{db_contract::types::ArcUuid7, user::status::UserStatus};

use super::User;

#[async_trait::async_trait]
pub trait UserRepositoryTrait: Send + Sync {
    async fn find_by_id(&self, id: ArcUuid7, without_trash: bool) -> anyhow::Result<Option<User>>;

    async fn find_by_username(
        &self,
        username: &str,
        without_trash: bool,
    ) -> anyhow::Result<Option<User>>;

    async fn find_by_email(&self, email: &str, without_trash: bool)
        -> anyhow::Result<Option<User>>;

    async fn create(&self, record: User) -> anyhow::Result<Option<User>>;

    async fn update(&self, id: &str, record: User) -> anyhow::Result<Option<User>>;

    async fn soft_delete(&self, id: ArcUuid7) -> anyhow::Result<()>;

    async fn delete(&self, id: ArcUuid7) -> anyhow::Result<()>;
}

pub trait UserTrait: Default + Clone {
    fn id(&self) -> ArcUuid7;
    fn username(&self) -> &String;
    fn email(&self) -> Option<&String>;
    fn reset_password(&self) -> bool;
    fn set_reset_password(&mut self, reset: bool);
    fn status(&self) -> UserStatus;
    fn set_status(&mut self, status: UserStatus);
    fn password(&self) -> &String;
    fn set_password(&mut self, password: &str);
    fn salt(&self) -> &String;
    fn set_salt(&mut self, salt: &str);
    fn login_attempt(&self) -> i64;
    fn set_login_attempt(&mut self, value: i64);
    fn is_sys_admin(&self) -> bool;
    fn set_is_sys_admin(&mut self, value: bool);
    fn last_login_at(&self) -> Option<&DateTime<Utc>>;
    fn set_last_login_at(&mut self, dt: DateTime<Utc>);
    fn created_at(&self) -> Option<&DateTime<Utc>>;
    fn updated_at(&self) -> Option<&DateTime<Utc>>;
    fn deleted_at(&self) -> Option<&DateTime<Utc>>;
    fn set_deleted_at(&mut self, dt: DateTime<Utc>);
}
