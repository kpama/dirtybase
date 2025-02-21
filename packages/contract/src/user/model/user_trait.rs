#![allow(dead_code)]

use crate::db::types::ArcUuid7;

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
