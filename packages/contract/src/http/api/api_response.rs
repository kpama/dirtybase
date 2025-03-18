use std::fmt::Debug;

use axum::{response::IntoResponse, Json};

#[derive(Debug, serde::Serialize)]
pub struct ApiResponse<D: serde::Serialize = ()> {
    data: Option<D>,
    error: Option<ApiError>,
}

impl<D: serde::Serialize> Default for ApiResponse<D> {
    fn default() -> Self {
        Self {
            data: None,
            error: None,
        }
    }
}

impl<D: serde::Serialize> ApiResponse<D> {
    pub fn new(data: Option<D>, error: Option<ApiError>) -> Self {
        Self { data, error }
    }

    pub fn success(data: D) -> Self {
        Self {
            data: Some(data),
            ..Self::default()
        }
    }

    pub fn error<E: Into<ApiError>>(error: E) -> Self {
        Self {
            error: Some(error.into()),
            ..Self::default()
        }
    }

    pub fn set_data(&mut self, data: D) {
        self.data = Some(data);
        self.error = None;
    }

    pub fn set_error<E: Into<ApiError>>(&mut self, error: E) {
        self.error = Some(error.into());
        self.data = None;
    }

    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }

    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }
}

impl<D: serde::Serialize + Debug, E: Debug> From<Result<D, E>> for ApiResponse<D> {
    fn from(value: Result<D, E>) -> Self {
        if value.is_ok() {
            Self::success(value.unwrap())
        } else {
            Self::error(format!("{:?}", value.unwrap_err()))
        }
    }
}

impl<D: serde::Serialize> From<Option<D>> for ApiResponse<D> {
    fn from(data: Option<D>) -> Self {
        Self::new(data, None)
    }
}

#[derive(Debug, Default, serde::Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub short_message: String,
}

impl ApiError {
    pub fn new(code: &str, message: &str, short_message: &str) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            short_message: short_message.into(),
        }
    }
}

impl From<String> for ApiError {
    fn from(value: String) -> Self {
        Self {
            code: value.clone().replace(' ', "_").to_lowercase(),
            message: value.clone(),
            short_message: value.clone(),
        }
    }
}

impl From<&str> for ApiError {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(value: anyhow::Error) -> Self {
        value.to_string().into()
    }
}

impl<D: serde::Serialize> IntoResponse for ApiResponse<D> {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}
