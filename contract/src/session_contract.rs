mod session;
mod session_data;
mod session_storage_resolver;
mod storage;

pub use session::*;
pub use session_data::*;
pub use session_storage_resolver::*;
pub use storage::*;

/// Default session life time is 2 hours
/// Value is in minutes
pub const DEFAULT_LIFETIME: u8 = 120; // 2 hours or 7200 seconds

pub fn generate_id() -> SessionId {
    SessionId::new()
}

pub mod prelude {
    pub use super::session::*;
    pub use super::session_data::*;
    pub use super::storage::*;
}
