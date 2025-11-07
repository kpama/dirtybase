use std::collections::HashMap;

use busybody::async_trait;
use dirtybase_contract::{
    auth_contract::{AuthUser, AuthUserPayload, AuthUserStorage, StorageResolver},
    db_contract::types::ArcUuid7,
};
use tokio::sync::RwLock;

#[derive(Default)]
pub struct AuthUserMemoryStorage {
    storage: RwLock<HashMap<ArcUuid7, AuthUser>>,
}

impl AuthUserMemoryStorage {
    pub const NAME: &str = "memory";

    pub fn new() -> Self {
        Self::default()
    }

    pub async fn register() {
        StorageResolver::register(Self::NAME, |mut resolver| async move {
            tracing::trace!("setting up memory auth storage");
            resolver.set_storage(AuthUserMemoryStorage::new());

            resolver
        })
        .await;
    }
}

#[async_trait]
impl AuthUserStorage for AuthUserMemoryStorage {
    async fn store(&self, mut payload: AuthUserPayload) -> Result<AuthUser, anyhow::Error> {
        let existing_id = payload.id.clone();
        let id = existing_id.unwrap_or_default();
        let mut w_lock = self.storage.write().await;
        if let Some(existing) = w_lock.get_mut(&id) {
            existing.merge(payload);
            existing.touch_updated_at();
            return Ok(existing.clone());
        } else {
            let mut new_user = AuthUser::default();
            payload.id = Some(id.clone());
            new_user.merge(payload);
            new_user.touch_created_at();
            new_user.touch_updated_at();
            w_lock.insert(id, new_user.clone());
            return Ok(new_user);
        }
    }
    async fn find_by_id(&self, id: ArcUuid7) -> Result<Option<AuthUser>, anyhow::Error> {
        let r_lock = self.storage.read().await;
        if let Some(user) = r_lock.get(&id) {
            return Ok(Some(user.clone()));
        }

        return Ok(None);
    }
    async fn find_by_username(&self, username: &str) -> Result<Option<AuthUser>, anyhow::Error> {
        let r_lock = self.storage.read().await;
        for entry in r_lock.values() {
            if entry.username_ref() == username {
                return Ok(Some(entry.clone()));
            }
        }

        return Ok(None);
    }
    async fn find_by_email_hash(&self, hash: &str) -> Result<Option<AuthUser>, anyhow::Error> {
        let r_lock = self.storage.read().await;
        for entry in r_lock.values() {
            if entry.email_hash_ref() == hash {
                return Ok(Some(entry.clone()));
            }
        }

        return Ok(None);
    }
    async fn delete(&self, id: ArcUuid7) -> Result<(), anyhow::Error> {
        let mut w_lock = self.storage.write().await;
        w_lock.remove(&id);
        Ok(())
    }
}
