mod domain;

use axum::extract::Request;
pub use domain::*;

pub fn clone_request<T>(req: &Request<T>) -> Request<()> {
    let mut r = Request::new(());
    *r.version_mut() = req.version();
    *r.method_mut() = req.method().clone();
    *r.uri_mut() = req.uri().clone();
    *r.headers_mut() = req.headers().clone();
    *r.extensions_mut() = req.extensions().clone();
    r
}
