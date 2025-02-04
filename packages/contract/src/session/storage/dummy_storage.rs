use crate::session::SessionData;

use super::{SessionId, SessionStorage};

#[derive(Default)]
pub(crate) struct DummyStorage;

#[async_trait::async_trait]
impl SessionStorage for DummyStorage {
    async fn open(&self, _id: SessionId) {
        log::debug!("dummy session storage open");
    }

    async fn store(&self, _id: SessionId, _value: SessionData) {
        log::debug!("dummy session storage store");
    }

    async fn get(&self, _id: &SessionId) -> Option<SessionData> {
        log::debug!("dummy session storage get");
        None
    }

    async fn remove(&self, _id: &SessionId) -> Option<SessionData> {
        log::debug!("dummy session storage remove");
        None
    }
    async fn gc(&self, _lifetime: i64) {
        log::debug!("dummy session storage clean expired");
    }
}
