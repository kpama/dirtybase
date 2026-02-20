use axum::http::StatusCode;
use axum::{
    body::Body,
    response::{IntoResponse, Response},
};

#[derive(Debug, PartialEq)]
pub struct GateResponse {
    allowed: bool,
    message: Option<String>,
    code: StatusCode,
}

impl Default for GateResponse {
    fn default() -> Self {
        Self {
            allowed: false,
            message: None,
            code: StatusCode::FORBIDDEN,
        }
    }
}

impl GateResponse {
    pub fn new(allowed: bool, code: StatusCode, message: Option<String>) -> Self {
        Self {
            allowed,
            code,
            message,
        }
    }

    pub fn allow() -> Self {
        Self {
            allowed: true,
            message: None,
            code: StatusCode::OK,
        }
    }

    pub fn deny() -> Self {
        Self::default()
    }

    pub fn allow_with_status(code: StatusCode, message: Option<String>) -> Self {
        Self {
            allowed: true,
            code,
            message,
        }
    }

    pub fn deny_with_status(code: StatusCode, message: Option<String>) -> Self {
        Self {
            allowed: false,
            code,
            message,
        }
    }
}

impl std::ops::Not for GateResponse {
    type Output = bool;
    fn not(self) -> Self::Output {
        self.allowed
    }
}

impl PartialEq<bool> for GateResponse {
    fn eq(&self, other: &bool) -> bool {
        self.allowed == *other
    }
}

impl PartialEq<GateResponse> for bool {
    fn eq(&self, other: &GateResponse) -> bool {
        *self == other.allowed
    }
}

impl From<bool> for GateResponse {
    fn from(value: bool) -> Self {
        if value { Self::allow() } else { Self::deny() }
    }
}

impl From<StatusCode> for GateResponse {
    fn from(code: StatusCode) -> Self {
        if code.is_success() {
            Self::allow_with_status(code, None)
        } else {
            Self::deny_with_status(code, None)
        }
    }
}

impl From<GateResponse> for Response {
    fn from(resp: GateResponse) -> Self {
        let body = if let Some(msg) = resp.message {
            Body::from(msg)
        } else {
            Body::empty()
        };

        (resp.code, body).into_response()
    }
}

impl IntoResponse for GateResponse {
    fn into_response(self) -> Response {
        self.into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_compare_allowed() {
        let resp = GateResponse::allow();

        assert!(resp == true);
    }

    #[test]
    fn test_compare_denied() {
        let resp = GateResponse::deny();

        assert!(resp == false);
    }
}
