use std::fmt::Debug;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

type MoreErrorData = serde_json::map::Map<String, serde_json::Value>;

#[derive(Debug, serde::Serialize)]
pub struct ApiResponse<D: serde::Serialize = ()> {
    data: Option<D>,
    error: Option<ApiError>,
    #[serde(skip)]
    status_code: Option<StatusCode>,
}

impl<D: serde::Serialize> Default for ApiResponse<D> {
    fn default() -> Self {
        Self {
            data: None,
            error: None,
            status_code: None,
        }
    }
}

impl<D: serde::Serialize> ApiResponse<D> {
    pub fn new(data: Option<D>, error: Option<ApiError>) -> Self {
        Self {
            data,
            error,
            status_code: None,
        }
    }

    pub fn success(data: D) -> Self {
        Self {
            data: Some(data),
            ..Self::default()
        }
    }

    pub fn created(data: D) -> Self {
        Self {
            data: Some(data),
            status_code: Some(StatusCode::CREATED),
            ..Self::default()
        }
    }

    pub fn error<E: Into<ApiError>>(error: E) -> Self {
        Self {
            error: Some(error.into()),
            ..Self::default()
        }
    }

    pub fn error_with_status<E: Into<ApiError>>(error: E, code: StatusCode) -> Self {
        Self {
            error: Some(error.into()),
            status_code: Some(code),
            ..Self::default()
        }
    }

    pub fn set_data(&mut self, data: D) {
        self.data = Some(data);
        self.error = None;
    }

    pub fn set_data_status(&mut self, data: D, code: StatusCode) {
        self.data = Some(data);
        self.error = None;
        self.status_code = Some(code);
    }

    pub fn set_error<E: Into<ApiError>>(&mut self, error: E) {
        self.error = Some(error.into());
        self.data = None;
    }

    pub fn set_error_and_status<E: Into<ApiError>>(&mut self, error: E, code: StatusCode) {
        self.error = Some(error.into());
        self.data = None;
        self.status_code = Some(code);
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
    pub more: Option<MoreErrorData>,
}

impl ApiError {
    pub fn new<T>(code: &str, message: &str, short_message: &str, more: Option<T>) -> Self
    where
        T: Serialize,
    {
        Self {
            code: code.into(),
            message: message.into(),
            short_message: short_message.into(),
            more: if let Some(m) = more {
                if let Ok(m) = serde_json::value::to_value(m) {
                    m.as_object().cloned()
                } else {
                    None
                }
            } else {
                None
            },
        }
    }
}

impl From<String> for ApiError {
    fn from(value: String) -> Self {
        Self {
            code: value.clone().replace(' ', "_").to_lowercase(),
            message: value.clone(),
            short_message: value.clone(),
            more: None,
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

impl<D: serde::Serialize, E> From<Result<Option<D>, E>> for ApiResponse<D>
where
    E: Into<ApiError>,
{
    fn from(value: Result<Option<D>, E>) -> Self {
        match value {
            Ok(Some(data)) => ApiResponse::success(data),
            Ok(None) => ApiResponse::error_with_status("resource not found", StatusCode::NOT_FOUND),
            _ => ApiResponse::error_with_status(
                "server process your request",
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        }
    }
}

impl<D: serde::Serialize> IntoResponse for ApiResponse<D> {
    fn into_response(self) -> axum::response::Response {
        if self.has_data() {
            (
                self.status_code.unwrap_or_else(|| StatusCode::OK),
                Json(self),
            )
                .into_response()
        } else {
            (
                self.status_code.unwrap_or_else(|| StatusCode::BAD_REQUEST),
                Json(self),
            )
                .into_response()
        }
    }
}

impl<D: serde::Serialize> Into<Response> for ApiResponse<D> {
    fn into(self) -> Response {
        self.into_response()
    }
}
