use axum::extract::{FromRequest, Request};
use axum_extra::extract::{Form, FormRejection};

#[derive(serde::Deserialize, Default)]
pub struct LoginCredential {
    username: Option<String>,
    email: Option<String>,
    password: String,
    #[serde(default)]
    remember_me: bool,
}

impl LoginCredential {
    pub fn username(&self) -> Option<&String> {
        self.username.as_ref()
    }

    pub fn email(&self) -> Option<&String> {
        self.email.as_ref()
    }

    pub fn password(&self) -> &str {
        self.password.as_str()
    }

    pub fn remember_me(&self) -> bool {
        self.remember_me
    }
}

pub struct LoginCredentialBuilder {
    credential: LoginCredential,
}

impl LoginCredentialBuilder {
    pub fn new() -> Self {
        Self {
            credential: Default::default(),
        }
    }

    pub fn username(&mut self, username: String) -> &mut Self {
        self.credential.username = Some(username);
        self
    }

    pub fn email(&mut self, email: String) -> &mut Self {
        self.credential.email = Some(email);
        self
    }

    pub fn password(&mut self, password: String) -> &mut Self {
        self.credential.password = password;
        self
    }
    pub fn remember_me(&mut self, remember_me: bool) -> &mut Self {
        self.credential.remember_me = remember_me;
        self
    }

    pub fn build(self) -> LoginCredential {
        self.credential
    }
}

impl<S: Send + Sync> FromRequest<S> for LoginCredential {
    type Rejection = FormRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Form::<LoginCredential>::from_request(req, state).await {
            Ok(c) => Ok(c.0),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_builder() {
        let cred = LoginCredentialBuilder::new()
            .username("foo".to_string())
            .password("john-doe!!".to_string())
            .build();

        assert_eq!(cred.password(), "john-doe!!");
        assert_eq!(cred.username.is_some(), true);
        assert_eq!(cred.username.unwrap().as_str(), "foo");
    }
}
