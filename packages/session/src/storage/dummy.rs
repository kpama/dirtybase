use anyhow::Ok;
use dirtybase_contract::session_contract::{
    SessionData, SessionId, SessionStorage, SessionStorageProvider, SessionStorageResolver,
};

pub const NAME: &str = "dummy";

#[derive(Default)]
pub struct DummyStorage;

impl DummyStorage {
    pub async fn register() {
        SessionStorageResolver::register(NAME, resolver).await;
    }
}

#[async_trait::async_trait]
impl SessionStorage for DummyStorage {
    async fn store(&self, _: SessionId, _: SessionData) {
        log::debug!("dummy session storage store");
    }

    async fn get(&self, _: &SessionId) -> SessionData {
        SessionData::new()
    }

    async fn remove(&self, _: &SessionId) -> Option<SessionData> {
        log::debug!("dummy session storage remove");
        None
    }
    async fn gc(&self, _: i64) {
        log::debug!("dummy session storage clean expired");
    }
}

pub async fn resolver(_: SessionStorageResolver) -> Result<SessionStorageProvider, anyhow::Error> {
    Ok(SessionStorageProvider::new(DummyStorage))
}
