use anyhow::anyhow;
use busybody::async_trait;
use dirtybase_contract::{
    auth_contract::{AuthUser, AuthUserPayload, AuthUserStorage, StorageResolver},
    db_contract::{
        base::{
            manager::Manager,
            table::{CREATED_AT_FIELD, UPDATED_AT_FIELD},
        },
        types::{ArcUuid7, ToColumnAndValue},
    },
};
use dirtybase_helper::time::current_datetime;

pub const AUTH_USER_TABLE: &str = "auth_users";

pub struct AuthUserDatabaseStorage {
    manager: Manager,
}

impl AuthUserDatabaseStorage {
    pub const NAME: &str = "database";

    pub fn new(manager: Manager) -> Self {
        Self { manager }
    }

    pub async fn register() {
        StorageResolver::register(Self::NAME, |mut resolver| async move {
            tracing::trace!("setting up database auth storage");
            if let Ok(manager) = resolver.context_ref().get::<Manager>().await {
                resolver.set_storage(AuthUserDatabaseStorage::new(manager));
            }

            resolver
        })
        .await;
    }
}

#[async_trait]
impl AuthUserStorage for AuthUserDatabaseStorage {
    async fn store(&self, payload: AuthUserPayload) -> Result<AuthUser, anyhow::Error> {
        let existing_id = payload.id.clone();
        let id = existing_id.clone().unwrap_or_default();

        let mut cv = payload.to_column_value().unwrap();

        if existing_id.is_some() {
            cv.insert(UPDATED_AT_FIELD.to_string(), current_datetime().into());
            self.manager
                .update(AUTH_USER_TABLE, cv, |q| {
                    q.is_eq("id", &id);
                })
                .await?;
        } else {
            cv.insert("id".to_string(), id.clone().into());
            cv.insert(CREATED_AT_FIELD.to_string(), current_datetime().into());
            self.manager.insert(AUTH_USER_TABLE, cv).await?;
        }
        match self
            .manager
            .select_from_table(AUTH_USER_TABLE, |q| {
                q.is_eq("id", id);
            })
            .fetch_one_to::<AuthUser>()
            .await
        {
            Ok(Some(user)) => Ok(user),
            Err(e) => Err(e),
            _ => Err(anyhow!("could not find user")),
        }
    }
    async fn find_by_id(&self, id: ArcUuid7) -> Result<Option<AuthUser>, anyhow::Error> {
        self.manager
            .select_from_table(AUTH_USER_TABLE, |q| {
                q.is_eq("id", id);
            })
            .fetch_one_to::<AuthUser>()
            .await
    }
    async fn find_by_username(&self, username: &str) -> Result<Option<AuthUser>, anyhow::Error> {
        self.manager
            .select_from_table(AUTH_USER_TABLE, |q| {
                q.is_eq("username", username);
            })
            .fetch_one_to::<AuthUser>()
            .await
    }
    async fn find_by_email_hash(&self, hash: &str) -> Result<Option<AuthUser>, anyhow::Error> {
        self.manager
            .select_from_table(AUTH_USER_TABLE, |q| {
                q.is_eq("email_hash", hash);
            })
            .fetch_one_to::<AuthUser>()
            .await
    }
    async fn delete(&self, id: ArcUuid7) -> Result<(), anyhow::Error> {
        self.manager
            .delete(AUTH_USER_TABLE, |q| {
                q.is_eq("id", id);
            })
            .await
    }
}

impl From<Manager> for AuthUserDatabaseStorage {
    fn from(manager: Manager) -> Self {
        Self::new(manager)
    }
}
