use axum::{
    extract::{rejection::PathRejection, FromRequestParts, Path, Request},
    http::request::Parts,
};
use serde::de::DeserializeOwned;

#[derive(Debug, Clone)]
pub struct PathValue {
    parts: Parts,
}

impl PathValue {
    pub fn from_request<T>(r: &Request<T>) -> Self {
        let mut req = Request::new(());
        *req.version_mut() = r.version();
        *req.method_mut() = r.method().clone();
        *req.uri_mut() = r.uri().clone();
        *req.headers_mut() = r.headers().clone();
        *req.extensions_mut() = r.extensions().clone();
        let (parts, _) = req.into_parts();

        Self { parts }
    }

    pub async fn get<T>(&mut self) -> Result<Path<T>, PathRejection>
    where
        T: DeserializeOwned + Send,
    {
        Path::<T>::from_request_parts(&mut self.parts, &()).await
    }
}
