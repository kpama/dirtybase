use anyhow::anyhow;
use busybody::async_trait;
use dirtybase_contract::{
    auth_contract::{AuthUser, AuthUserPayload, AuthUserStorage},
    db_contract::{base::manager::Manager, types::ArcUuid7},
};

use crate::AUTH_USER_TABLE;

pub struct AuthUserDatabaseStorage {
    manager: Manager,
}

impl AuthUserDatabaseStorage {
    pub fn new(manager: Manager) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl AuthUserStorage for AuthUserDatabaseStorage {
    async fn store(&self, mut payload: AuthUserPayload) -> Result<AuthUser, anyhow::Error> {
        let existing_id = payload.id.clone();
        let id = existing_id.clone().unwrap_or_default();

        if existing_id.is_some() {
            self.manager
                .update(AUTH_USER_TABLE, payload, |q| {
                    q.eq("id", &id);
                })
                .await?;
        } else {
            payload.id = Some(id.clone());
            self.manager.insert(AUTH_USER_TABLE, payload).await?;
        }
        match self
            .manager
            .select_from_table(AUTH_USER_TABLE, |q| {
                q.eq("id", id);
            })
            .fetch_one_to::<AuthUser>()
            .await
        {
            Ok(Some(user)) => Ok(user),
            Err(e) => Err(e),
            _ => Err(anyhow!("could not find user")),
        }
    }
    async fn exists_by_username(&self, username: &str) -> bool {
        let result = self.find_by_username(username).await;

        match result {
            Ok(option) => match option {
                Some(_) => true,
                None => false,
            },
            Err(_) => false,
        }
    }
    async fn find_by_id(&self, id: ArcUuid7) -> Result<Option<AuthUser>, anyhow::Error> {
        self.manager
            .select_from_table(AUTH_USER_TABLE, |q| {
                q.eq("id", id);
            })
            .fetch_one_to::<AuthUser>()
            .await
    }
    async fn find_by_username(&self, username: &str) -> Result<Option<AuthUser>, anyhow::Error> {
        self.manager
            .select_from_table(AUTH_USER_TABLE, |q| {
                q.eq("username", username);
            })
            .fetch_one_to::<AuthUser>()
            .await
    }
    async fn find_by_email_hash(&self, hash: &str) -> Result<Option<AuthUser>, anyhow::Error> {
        self.manager
            .select_from_table(AUTH_USER_TABLE, |q| {
                q.eq("email_hash", hash);
            })
            .fetch_one_to::<AuthUser>()
            .await
    }
    async fn delete(&self, id: ArcUuid7) -> Result<(), anyhow::Error> {
        self.manager
            .delete(AUTH_USER_TABLE, |q| {
                q.eq("id", id);
            })
            .await
    }
}

impl From<Manager> for AuthUserDatabaseStorage {
    fn from(manager: Manager) -> Self {
        Self::new(manager)
    }
}
