use dirtybase_contract::{
    db_contract::types::{ArcUuid7, StringField},
    prelude::{Context, axum_extra::extract::cookie::Cookie},
    session_contract::Session,
};
use dirtybase_helper::random::random_bytes_hex;
use serde::{Deserialize, Serialize};

const SESSION_KEY: &str = "_auth";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AuthSession {
    hash: Option<StringField>, // Random hash
    ck: Option<StringField>,   // Cookie key
    rd: StringField,           // Redirect url
    user: Option<ArcUuid7>,    // User Id
}

impl AuthSession {
    pub fn new(user: Option<ArcUuid7>) -> Self {
        Self {
            hash: Some(random_bytes_hex(16).into()),
            ck: Some(random_bytes_hex(4).into()),
            rd: "/".to_string().into(),
            user,
        }
    }

    pub async fn save(&self, session: &Session) {
        session.put(SESSION_KEY, &self).await;
    }

    pub async fn to_cookie(&self, session: &Session) -> Cookie<'static> {
        self.save(session).await;
        let hash = self
            .hash
            .clone()
            .unwrap_or_else(|| random_bytes_hex(16).into());
        let key = self
            .ck
            .clone()
            .unwrap_or_else(|| random_bytes_hex(4).into());
        session.make_session_cookie(&key, &hash)
    }

    pub async fn delete(&self, session: Session, ctx: &Context) -> Session {
        session.invalidate(ctx).await
    }

    pub async fn from_session(session: &Session) -> Self {
        session.get::<Self>(SESSION_KEY).await.unwrap_or_default()
    }

    pub fn hash(&self) -> Option<&String> {
        self.hash.as_deref()
    }

    pub fn cookie_key(&self) -> Option<&String> {
        self.ck.as_deref()
    }

    pub fn user_id(&self) -> Option<&ArcUuid7> {
        self.user.as_ref()
    }

    pub fn redirect(&self) -> &String {
        self.rd.as_ref()
    }

    pub fn set_redirect(&mut self, url: &str) {
        self.rd = url.to_string().into();
    }
}
