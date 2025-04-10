use dirtybase_contract::session_contract::{SessionData, SessionId, SessionStorage};

use crate::SessionStorageResolver;

pub const NAME: &'static str = "dummy";

#[derive(Default)]
pub struct DummyStorage;

#[async_trait::async_trait]
impl SessionStorage for DummyStorage {
    async fn store(&self, _id: SessionId, _value: SessionData) {
        log::debug!("dummy session storage store");
    }

    async fn get(&self, _id: &SessionId) -> SessionData {
        SessionData::new()
    }

    async fn remove(&self, _id: &SessionId) -> Option<SessionData> {
        log::debug!("dummy session storage remove");
        None
    }
    async fn gc(&self, _lifetime: i64) {
        log::debug!("dummy session storage clean expired");
    }
}

pub async fn resolver(mut resolver: SessionStorageResolver) -> SessionStorageResolver {
    resolver.set_storage(DummyStorage::default());
    resolver
}
