use cookie::Cookie;
use serde::de::DeserializeOwned;

use super::{SessionId, SessionStorage, SessionStorageProvider};

#[derive(Clone)]
pub struct Session {
    id: SessionId,
    storage: SessionStorageProvider,
    lifetime: i64,
}

impl Session {
    pub(crate) async fn new(id: SessionId, storage: SessionStorageProvider) -> Self {
        Self {
            id,
            storage,
            lifetime: 60,
        }
    }

    pub async fn init(
        id: SessionId,
        storage: SessionStorageProvider,
        lifetime: i64,
        fingerprint: &str,
    ) -> Self {
        let mut session = Self::new(id, storage).await;
        session.lifetime = lifetime;
        let is_valid = if fingerprint.is_empty() || fingerprint != session.fingerprint().await {
            false
        } else {
            !(session.has_expired(lifetime).await)
        };

        if is_valid {
            session.touch().await;
        } else {
            session = session.invalidate().await;
            session.set_fingerprint(fingerprint).await;
        }

        session
    }

    pub async fn get<T: DeserializeOwned>(&self, name: &str) -> Option<T> {
        let bucket = self.storage.get(&self.id).await;
        if let Some(value) = bucket.get(name) {
            return serde_json::from_value(value).ok();
        }

        None
    }

    pub async fn put<V: serde::Serialize>(&self, name: &str, value: V) {
        let bucket = self.storage.get(&self.id).await;
        bucket.add(
            name.to_string(),
            serde_json::to_value(&value).unwrap_or_default(),
        );
        bucket.touch(self.lifetime);
        self.storage.store(self.id.clone(), bucket).await;
    }

    pub fn id(&self) -> SessionId {
        self.id.clone()
    }

    pub async fn touch(&self) {
        let bucket = self.storage.get(&self.id).await;
        bucket.touch(self.lifetime);
        self.storage.store(self.id.clone(), bucket).await;
    }

    pub async fn has_expired(&self, lifetime: i64) -> bool {
        let bucket = self.storage.get(&self.id).await;
        bucket.has_expired(lifetime)
    }

    pub async fn delete(self) {
        _ = self.storage.remove(&self.id).await;
    }

    pub async fn invalidate(self) -> Self {
        let fingerprint = self.fingerprint().await;
        let lifetime = self.lifetime;

        self.storage.remove(&self.id).await;

        let mut instance = Self::new(SessionId::new(), self.storage).await;
        instance.lifetime = lifetime;
        instance.set_fingerprint(&fingerprint).await;

        instance
    }

    /// Creates a cookie that has the same lifetime as the session
    pub fn make_session_cookie<V>(&self, name: &str, value: V) -> Cookie<'static>
    where
        V: ToString,
    {
        let mut cookie = Cookie::new(name.to_string(), value.to_string());
        let mut ts = cookie::time::OffsetDateTime::now_utc();
        ts += cookie::time::Duration::minutes(self.lifetime);

        cookie.set_path("/");
        cookie.set_expires(ts);
        cookie
    }

    async fn fingerprint(&self) -> String {
        println!(
            "getting finger print: {:?}",
            self.get::<String>("_fp").await
        );
        self.get("_fp").await.unwrap_or_default()
    }

    async fn set_fingerprint(&self, value: &str) {
        self.put("_fp", value).await
    }
}
