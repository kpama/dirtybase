use cookie::Cookie;
use serde::de::DeserializeOwned;

use crate::{http_contract::HttpContext, prelude::Context, session_contract::SessionData};

use super::{SessionId, SessionStorage, SessionStorageProvider};

#[derive(Clone)]
pub struct Session {
    id: SessionId,
    storage: SessionStorageProvider,
    data: SessionData,
    lifetime: i64,
}

impl Session {
    pub(crate) async fn new(
        storage: SessionStorageProvider,
        lifetime: i64,
        id: Option<SessionId>,
        data: Option<SessionData>,
    ) -> Self {
        let d = if let Some(d) = data {
            d
        } else if id.is_some() {
            storage.get(id.as_ref().unwrap()).await
        } else {
            SessionData::new()
        };

        Self {
            id: id.unwrap_or_default(),
            data: d,
            storage,
            lifetime,
        }
    }

    pub async fn init(
        id: Option<SessionId>,
        storage: SessionStorageProvider,
        lifetime: i64,
        context: &Context,
    ) -> Self {
        let http_context = context.get::<HttpContext>().await.unwrap();
        let fingerprint = http_context.fingerprint();
        let is_new = id.is_none();
        let mut session = Self::new(storage, lifetime, id, None).await;
        let is_valid = if is_new {
            true
        } else if fingerprint != session.fingerprint().await {
            false
        } else {
            !(session.has_expired(lifetime).await)
        };

        if is_valid {
            session.touch().await;
            context.set(session.clone()).await;
        } else {
            session = session.invalidate(context).await;
        }

        session.set_fingerprint(&fingerprint).await;

        session
    }

    pub async fn get<T: DeserializeOwned>(&self, name: &str) -> Option<T> {
        if let Some(value) = self.data.get(name) {
            return serde_json::from_value(value).ok();
        }

        None
    }

    pub async fn put<V: serde::Serialize>(&self, name: &str, value: V) {
        self.data.add(name, value);
        self.data.touch(self.lifetime);
    }

    pub async fn remove(&self, key: &str) {
        self.data.delete(key);
        self.data.touch(self.lifetime);
    }

    pub fn id(&self) -> SessionId {
        self.id.clone()
    }

    pub async fn touch(&self) {
        self.data.touch(self.lifetime);
    }

    pub async fn has_expired(&self, lifetime: i64) -> bool {
        self.data.has_expired(lifetime)
    }

    pub async fn delete(self) {
        _ = self.storage.remove(&self.id).await;
    }

    pub async fn save(&self) {
        self.storage.store(self.id(), self.data.clone()).await;
    }

    pub async fn invalidate(self, ctx: &Context) -> Self {
        let fingerprint = self.fingerprint().await;

        self.storage.remove(&self.id).await;

        let instance = Self::new(self.storage, self.lifetime, None, Some(SessionData::new())).await;

        if !fingerprint.is_empty() {
            instance.set_fingerprint(&fingerprint).await;
        }

        ctx.set(instance.clone()).await;

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
        self.get::<String>("_fp").await.unwrap_or_default()
    }

    async fn set_fingerprint(&self, value: &str) {
        self.put("_fp", value).await
    }
}
