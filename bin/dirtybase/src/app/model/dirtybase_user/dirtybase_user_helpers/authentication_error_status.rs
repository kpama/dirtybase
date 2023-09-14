use std::fmt::Display;

use crate::http::http_helpers::ApiError;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "code")]
pub enum AuthenticationErrorStatus {
    UserNotFound,
    PasswordIncorrect,
    AccountSuspended,
    AccountInactive,
    AuthenticationFailed,
    CredentialIncorrect,
}

pub const AUTH_ERROR_USER_NOT_FOUND: &str = "User not found";
pub const AUTH_ERROR_PASSWORD_INCORRECT: &str = "Password incorrect";
pub const AUTH_ERROR_ACCOUNT_SUSPENDED: &str = "Account suspended";
pub const AUTH_ERROR_ACCOUNT_INACTIVE: &str = "Account inactive";
pub const AUTH_ERROR_AUTHENTICATION_FAILED: &str = "Authentication failed";
pub const AUTH_ERROR_CREDENTIAL_INCORRECT: &str = "Login credential incorrect";

impl Display for AuthenticationErrorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserNotFound => write!(f, "{}", AUTH_ERROR_USER_NOT_FOUND),
            Self::PasswordIncorrect => write!(f, "{}", AUTH_ERROR_PASSWORD_INCORRECT),
            Self::AccountSuspended => write!(f, "{}", AUTH_ERROR_ACCOUNT_SUSPENDED),
            Self::AccountInactive => write!(f, "{}", AUTH_ERROR_ACCOUNT_INACTIVE),
            Self::AuthenticationFailed => write!(f, "{}", AUTH_ERROR_AUTHENTICATION_FAILED),
            Self::CredentialIncorrect => write!(f, "{}", AUTH_ERROR_CREDENTIAL_INCORRECT),
        }
    }
}

impl From<AuthenticationErrorStatus> for ApiError {
    fn from(value: AuthenticationErrorStatus) -> Self {
        // `Foo bar` becomes `foo_bar`
        let code = value
            .to_string()
            .to_lowercase()
            .split(' ')
            .collect::<Vec<&str>>()
            .join("_");

        ApiError::new(
            &code,
            value.to_string().as_str(),
            value.to_string().as_str(),
        )
    }
}
