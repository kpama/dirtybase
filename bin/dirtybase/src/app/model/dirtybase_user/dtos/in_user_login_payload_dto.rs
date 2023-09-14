use busybody::Injectable;

use crate::app::model::dirtybase_user::dirtybase_user_helpers::authentication_error_status::AuthenticationErrorStatus;

// TODO: remove serde::Serialization. This data should never leave the backend
/// Incoming DTO when the user is attempting to login
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct UserLoginPayload {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: String,
}

impl UserLoginPayload {
    pub(crate) fn cache_id(&self) -> String {
        if let Some(username) = self.username.as_ref() {
            username.clone()
        } else if let Some(email) = self.email.as_ref() {
            email.clone()
        } else {
            "".into()
        }
    }

    pub(crate) fn validate(&self) -> Result<bool, AuthenticationErrorStatus> {
        if (self.username.is_none() && self.email.is_none())
            || (self.username.as_ref().unwrap().is_empty()
                && self.email.as_ref().unwrap().is_empty())
        {
            return Err(AuthenticationErrorStatus::CredentialIncorrect);
        }

        if self.password.is_empty() {
            return Err(AuthenticationErrorStatus::PasswordIncorrect);
        }

        Ok(true)
    }
}

impl Default for UserLoginPayload {
    fn default() -> Self {
        Self {
            username: None,
            email: None,
            password: "".to_string(),
        }
    }
}

#[busybody::async_trait]
impl Injectable for UserLoginPayload {
    async fn inject(container: &busybody::ServiceContainer) -> Self {
        container.proxy_value().unwrap_or_default()
    }
}
