use std::sync::Arc;

use axum::{
    body::Body,
    http::{self, uri::Authority, HeaderMap, Request},
};

pub fn full_request_url(req: &Request<Body>) -> String {
    // TODO: Look into why axum is not giving us the full URL via Uri
    if req.uri().scheme().is_some() {
        return req.uri().to_string();
    }

    if let Some(domain) = host_from_request(req) {
        return format!("{}{}", domain, req.uri()); // domain.com/path?query
    }

    "/".to_string()
}

/// Given the request, this function will try to pluck out the current host
///
/// Most of the log here was taken from: Axum-extra "Host" extension
pub fn host_from_request<T>(req: &Request<T>) -> Option<Arc<String>> {
    // logic taken from "axum-extra/src/extract/host.rst"
    if let Some(host) = parse_forwarded(req.headers()) {
        return Some(Arc::new(host.to_string()));
    }

    // check x-forward header
    if let Some(host) = req
        .headers()
        .get("X-Forwarded-Host")
        .and_then(|h| h.to_str().ok())
    {
        return Some(Arc::new(host.to_string()));
    }

    if let Some(host) = req
        .headers()
        .get(http::header::HOST)
        .and_then(|h| h.to_str().ok())
    {
        return Some(Arc::new(host.to_string()));
    }

    // logic taken from "axum-extra/src/extract/host.rst"
    if let Some(authority) = req.uri().authority() {
        return Some(Arc::new(parse_authority(authority).to_string()));
    }

    None
}

// copied from: axum-extra @ /src/extract/host.rs
fn parse_forwarded(headers: &HeaderMap) -> Option<&str> {
    // if there are multiple `Forwarded` `HeaderMap::get` will return the first one
    let forwarded_values = headers.get(http::header::FORWARDED)?.to_str().ok()?;

    // get the first set of values
    let first_value = forwarded_values.split(',').next()?;

    // find the value of the `host` field
    first_value.split(';').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        key.trim()
            .eq_ignore_ascii_case("host")
            .then(|| value.trim().trim_matches('"'))
    })
}

// copied from: axum-extra @ /src/extract/host.rs
fn parse_authority(auth: &Authority) -> &str {
    auth.as_str()
        .rsplit('@')
        .next()
        .expect("split always has at least 1 item")
}
