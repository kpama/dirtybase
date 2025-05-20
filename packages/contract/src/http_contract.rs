mod http_bind;
mod http_context;
mod path_value;
mod router_builder;
mod router_manager;
mod web_middleware_manager;

pub mod api;
pub mod axum;

use std::sync::Arc;

pub use http_bind::*;
pub use http_context::*;
pub use named_routes_axum;
pub use router_builder::*;
pub use router_manager::*;
pub use web_middleware_manager::*;
pub type WebAppState = Arc<busybody::ServiceContainer>;

pub mod prelude {
    pub use super::router_manager::*;
    pub use super::web_middleware_manager::*;
    pub use super::WebAppState;
    pub use axum::body::*;
    pub use axum::extract::Request;
    pub use axum::extract::*;
    pub use axum::http::*;
    pub use axum::middleware::*;
    pub use axum::response::Form;
    pub use axum::response::Html;
    pub use axum::response::IntoResponse;
    pub use axum::response::Json;
    pub use axum_extra;
    pub use axum_extra::headers::authorization::*;
    pub use axum_extra::headers::Header;
    pub use named_routes_axum::*;
    pub use validator::Validate;
}
