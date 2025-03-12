use std::sync::Arc;

use crate::db::types::ArcUuid7;

use super::model::{User, UserRepositoryTrait};

#[derive(Clone)]
pub struct UserProviderService(Arc<Box<dyn UserRepositoryTrait + 'static>>);

#[async_trait::async_trait]
impl UserRepositoryTrait for UserProviderService {
    async fn find_by_id(&self, id: ArcUuid7, without_trash: bool) -> anyhow::Result<Option<User>> {
        self.0.find_by_id(id, without_trash).await
    }

    async fn find_by_username(
        &self,
        username: &str,
        without_trash: bool,
    ) -> anyhow::Result<Option<User>> {
        self.0.find_by_username(username, without_trash).await
    }

    async fn find_by_email(
        &self,
        email: &str,
        without_trash: bool,
    ) -> anyhow::Result<Option<User>> {
        self.0.find_by_email(email, without_trash).await
    }

    async fn create(&self, record: User) -> anyhow::Result<Option<User>> {
        self.0.create(record).await
    }

    async fn update(&self, id: &str, record: User) -> anyhow::Result<Option<User>> {
        self.0.update(id, record).await
    }

    async fn soft_delete(&self, id: ArcUuid7) -> anyhow::Result<()> {
        self.0.soft_delete(id).await
    }

    async fn delete(&self, id: ArcUuid7) -> anyhow::Result<()> {
        self.0.soft_delete(id).await
    }
}
