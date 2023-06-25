use std::fmt::Display;

use crate::http::http_helpers::ApiError;

#[derive(Debug, serde::Serialize)]
#[serde(tag = "code")]
pub enum AuthenticationErrorStatus {
    UserNotFound,
    PasswordIncorrect,
    AccountSuspended,
    AccountInactive,
    AuthenticationFailed,
}

pub const AUTH_ERROR_USER_NOT_FOUND: &str = "User not found";
pub const AUTH_ERROR_PASSWORD_INCORRECT: &str = "Password incorrect";
pub const AUTH_ERROR_ACCOUNT_SUSPENDED: &str = "Account suspended";
pub const AUTH_ERROR_ACCOUNT_INACTIVE: &str = "Account inactive";
pub const AUTH_ERROR_AUTHENTICATION_FAILED: &str = "Authentication failed";

impl Display for AuthenticationErrorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserNotFound => write!(f, "{}", AUTH_ERROR_USER_NOT_FOUND),
            Self::PasswordIncorrect => write!(f, "{}", AUTH_ERROR_PASSWORD_INCORRECT),
            Self::AccountSuspended => write!(f, "{}", AUTH_ERROR_ACCOUNT_SUSPENDED),
            Self::AccountInactive => write!(f, "{}", AUTH_ERROR_ACCOUNT_INACTIVE),
            Self::AuthenticationFailed => write!(f, "{}", AUTH_ERROR_AUTHENTICATION_FAILED),
        }
    }
}

impl From<AuthenticationErrorStatus> for ApiError {
    fn from(value: AuthenticationErrorStatus) -> Self {
        match &value {
            AuthenticationErrorStatus::UserNotFound => ApiError::new(
                "user_not_found",
                value.to_string().as_str(),
                value.to_string().as_str(),
            ),
            AuthenticationErrorStatus::PasswordIncorrect => ApiError::new(
                "password_incorrent",
                value.to_string().as_str(),
                value.to_string().as_str(),
            ),
            AuthenticationErrorStatus::AccountSuspended => ApiError::new(
                "password_incorrent",
                value.to_string().as_str(),
                value.to_string().as_str(),
            ),
            AuthenticationErrorStatus::AccountInactive => ApiError::new(
                "account_inactive",
                value.to_string().as_str(),
                value.to_string().as_str(),
            ),
            AuthenticationErrorStatus::AuthenticationFailed => ApiError::new(
                "authentication_failed",
                value.to_string().as_str(),
                value.to_string().as_str(),
            ),
        }
    }
}
