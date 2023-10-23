use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock},
};

use self::base::{
    connection::ConnectionPoolTrait,
    schema::{ClientType, DatabaseKind},
};

pub mod base;
pub mod config;
pub mod entity;
pub mod event;
pub mod event_handler;
pub mod migration;

pub use dirtybase_db_macro as macros;
pub use dirtybase_db_types;

pub(crate) static LAST_WRITE_TS: OnceLock<RwLock<HashMap<DatabaseKind, i64>>> = OnceLock::new();
pub type ConnectionsType = HashMap<DatabaseKind, HashMap<ClientType, Box<dyn ConnectionPoolTrait>>>;
