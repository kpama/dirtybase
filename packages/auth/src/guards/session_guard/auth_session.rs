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
    actor: Option<ArcUuid7>,   // Actor Id
}

impl AuthSession {
    pub fn new(actor: Option<ArcUuid7>) -> Self {
        Self {
            hash: Some(random_bytes_hex(16).into()),
            ck: Some(random_bytes_hex(4).into()),
            rd: "/".to_string().into(),
            actor,
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
        let mut cookie = session.make_session_cookie(&key, &hash);
        cookie.set_http_only(true);
        cookie
    }

    pub async fn delete(&self, session: Session, ctx: &Context) -> Session {
        session.invalidate(ctx).await
    }

    pub async fn from_session(session: &Session) -> Option<Self> {
        session.get::<Self>(SESSION_KEY).await
    }

    pub fn hash(&self) -> Option<&StringField> {
        self.hash.as_ref()
    }

    pub fn cookie_key(&self) -> Option<&StringField> {
        self.ck.as_ref()
    }

    pub fn actor_id(&self) -> Option<&ArcUuid7> {
        self.actor.as_ref()
    }

    pub fn redirect(&self) -> &str {
        self.rd.as_ref()
    }

    pub fn set_redirect(&mut self, url: &str) {
        self.rd = url.to_string().into();
    }
}
