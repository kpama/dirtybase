mod config;
mod session;
mod session_data;
mod storage;

pub use config::*;
pub use session::*;
pub use session_data::*;
pub use storage::*;

pub const DEFAULT_LIFETIME: u8 = 120; // 2 hours or 7200 seconds

pub fn generate_id() -> SessionId {
    SessionId::new()
}

pub mod prelude {
    pub use super::config::*;
    pub use super::session::*;
    pub use super::session_data::*;
    pub use super::storage::*;
}
