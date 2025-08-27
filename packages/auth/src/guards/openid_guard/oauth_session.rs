use std::fmt::Debug;

use dirtybase_contract::{db_contract::types::StringField, session_contract::Session};
use dirtybase_helper::random::random_bytes_hex;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenIdProvider(StringField);

impl OpenIdProvider {
    pub fn id(&self) -> &str {
        self.as_ref()
    }
}

impl AsRef<str> for OpenIdProvider {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for OpenIdProvider {
    type Error = String;
    fn try_from(name: &str) -> Result<Self, Self::Error> {
        let clean = name.to_lowercase().replace(' ', "");
        if clean.is_empty() {
            return Err("Provider name cannot be empty".to_string());
        }

        Ok(Self(clean.into()))
    }
}

impl TryFrom<String> for OpenIdProvider {
    type Error = String;
    fn try_from(name: String) -> Result<Self, Self::Error> {
        name.as_str().try_into()
    }
}

impl Debug for OpenIdProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl Default for OpenIdProvider {
    fn default() -> Self {
        OpenIdProvider("unknown".to_string().into())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OauthSession {
    provider: OpenIdProvider,
    state: StringField,
    nonce: StringField,
}

pub const SESSION_KEY: &str = "_oauth";

impl OauthSession {
    pub fn new(provider: OpenIdProvider) -> Self {
        Self {
            provider,
            state: random_bytes_hex(16).into(),
            nonce: random_bytes_hex(16).into(),
        }
    }

    pub async fn save(&self, session: &Session) {
        session.put(SESSION_KEY, &self).await;
    }

    pub async fn delete(&self, session: &Session) {
        session.remove(SESSION_KEY).await;
    }

    pub fn provider(&self) -> &OpenIdProvider {
        &self.provider
    }

    pub fn state(&self) -> &str {
        self.state.as_str()
    }

    pub fn nonce(&self) -> &str {
        self.nonce.as_str()
    }

    pub async fn from_session(session: &Session) -> Self {
        session.get::<Self>(SESSION_KEY).await.unwrap_or_default()
    }
}
