use std::ops::Deref;

use crate::db::types::ArcUuid7;

use super::{AuthUser, AuthUserPayload};

#[async_trait::async_trait]
pub trait AuthUserStorage: Send + Sync {
    async fn store(&self, payload: AuthUserPayload) -> Result<AuthUser, anyhow::Error>;
    async fn find_by_id(&self, id: ArcUuid7) -> Result<Option<AuthUser>, anyhow::Error>;
    async fn find_by_username(&self, username: ArcUuid7)
        -> Result<Option<AuthUser>, anyhow::Error>;
    async fn find_by_email_hash(&self, hash: &str) -> Result<Option<AuthUser>, anyhow::Error>;
    async fn delete(&self, id: ArcUuid7) -> Result<(), anyhow::Error>;
}

pub struct AuthUserStorageProvider(Box<dyn AuthUserStorage>);

impl AuthUserStorageProvider {
    pub fn new<T>(inner: T) -> Self
    where
        T: AuthUserStorage + 'static,
    {
        Self(Box::new(inner))
    }

    pub fn from<S>(storage: S) -> Self
    where
        S: AuthUserStorage + 'static,
    {
        Self(Box::new(storage))
    }
}

impl Deref for AuthUserStorageProvider {
    type Target = Box<dyn AuthUserStorage>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait::async_trait]
impl AuthUserStorage for AuthUserStorageProvider {
    async fn store(&self, payload: AuthUserPayload) -> Result<AuthUser, anyhow::Error> {
        self.0.store(payload).await
    }
    async fn find_by_id(&self, id: ArcUuid7) -> Result<Option<AuthUser>, anyhow::Error> {
        self.0.find_by_id(id).await
    }
    async fn find_by_username(
        &self,
        username: ArcUuid7,
    ) -> Result<Option<AuthUser>, anyhow::Error> {
        self.0.find_by_username(username).await
    }
    async fn find_by_email_hash(&self, hash: &str) -> Result<Option<AuthUser>, anyhow::Error> {
        self.0.find_by_email_hash(hash).await
    }
    async fn delete(&self, id: ArcUuid7) -> Result<(), anyhow::Error> {
        self.0.delete(id).await
    }
}
