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
            code: value.clone().replace(' ', "_").to_ascii_lowercase(),
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
