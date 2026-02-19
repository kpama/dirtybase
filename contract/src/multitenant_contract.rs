pub mod model;
mod storage;
mod tenant_context;
mod tenant_resolved_middleware;

pub use storage::*;
pub use tenant_context::*;
pub use tenant_resolved_middleware::*;

pub const TENANT_ID_HEADER: &str = "X-Tenant-ID";
pub const TENANT_TOKEN_HEADER: &str = "X-Tenant-Token";
pub const TENANT_ID_QUERY_STRING: &str = "tenant-id";
pub const TENANT_TOKEN_QUERY_STRING: &str = "tenant-tk";
pub const TENANT_ID_COOKIE: &str = "tenant-id";
