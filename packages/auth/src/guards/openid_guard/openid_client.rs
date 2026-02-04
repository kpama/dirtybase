use std::{collections::HashSet, sync::Arc};

use dirtybase_helper::random::random_bytes_hex;
use serde::{Deserialize, Serialize};

pub struct OpenIdClient {
    client_id: Arc<String>,
    _client_secret: Arc<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RedirectInfo {
    client_id: String,
    nonce: String,
    state: String,
    scope: HashSet<String>,
    redirect_uri: String,
    to: String,
    flow: AuthFlow,
}

impl RedirectInfo {
    pub fn new(endpoint: &str, redirect_uri: &str) -> Self {
        Self {
            to: endpoint.to_string(),
            redirect_uri: redirect_uri.to_string(),
            ..Default::default()
        }
    }

    pub fn set_client_id(&mut self, client_id: &str) -> &mut Self {
        self.client_id = client_id.to_string();
        self
    }

    pub fn append_scope(&mut self, scope: &[&str]) -> &mut Self {
        for a_scope in scope {
            self.scope.insert(a_scope.to_string());
        }

        self
    }

    pub fn set_redirect_uri(&mut self, uri: &str) -> &mut Self {
        self.redirect_uri = uri.to_string();

        self
    }

    pub fn set_endpoint(&mut self, endpoint: &str) -> &mut Self {
        self.to = endpoint.to_string();
        self
    }

    pub fn set_flow(&mut self, flow: AuthFlow) -> &mut Self {
        self.flow = flow;
        self
    }
}

impl Default for RedirectInfo {
    fn default() -> Self {
        let mut scope = HashSet::new();
        scope.extend(vec!["openid".to_string(), "profile".to_string()]);

        RedirectInfo {
            nonce: random_bytes_hex(8).to_string(),
            state: random_bytes_hex(16).to_string(),
            scope,
            redirect_uri: String::new(),
            to: String::new(),
            client_id: String::new(),
            flow: AuthFlow::CodeFlow,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum AuthFlow {
    #[default]
    CodeFlow,
    ImplicitFlow,
    HybridFlow,
}

impl RedirectInfo {
    pub fn url(&self) -> String {
        let response_type = match self.flow {
            AuthFlow::CodeFlow => "code",
            AuthFlow::ImplicitFlow => "id_token token",
            AuthFlow::HybridFlow => "code id_token token",
        };

        let scope = &self
            .scope
            .iter()
            .fold(Vec::with_capacity(self.scope.len()), |mut vec, val| {
                vec.push(val.as_str());
                vec
            })
            .join(" ");

        let params = &[
            ("response_type", response_type),
            ("client_id", &self.client_id),
            ("scope", scope),
            ("redirect_uri", &self.redirect_uri),
            ("state", &self.state),
            ("nonce", &self.nonce),
        ];

        format!(
            "{}?{}",
            self.to,
            serde_urlencoded::to_string(params).unwrap_or_default()
        )
    }
}

impl OpenIdClient {
    pub fn new(client_id: &str, client_secret: &str) -> Self {
        Self {
            client_id: client_id.to_string().into(),
            _client_secret: client_secret.to_string().into(),
        }
    }

    pub fn build_redirect_with(&self, mut info: RedirectInfo) -> RedirectInfo {
        info.client_id = self.client_id.as_str().to_string();

        info
    }

    pub fn build_redirect(
        &self,
        endpoint: &str,
        redirect_uri: &str,
        scope: &[&str],
    ) -> RedirectInfo {
        let mut info = RedirectInfo::new(endpoint, redirect_uri);
        info.append_scope(scope);

        info
    }
}

#[cfg(test)]
mod test {

    //
}
