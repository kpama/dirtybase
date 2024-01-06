mod column_value_builder;
mod table_entity;

pub mod base;
pub mod config;
pub mod event;
pub mod event_handler;
pub mod field_values;
pub mod migration;
pub mod types;

use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock},
};

pub use anyhow;
use base::{
    connection::ConnectionPoolTrait,
    schema::{ClientType, DatabaseKind},
};
pub use column_value_builder::*;
pub use table_entity::*;

pub(crate) static LAST_WRITE_TS: OnceLock<RwLock<HashMap<DatabaseKind, i64>>> = OnceLock::new();
pub type ConnectionsType = HashMap<DatabaseKind, HashMap<ClientType, Box<dyn ConnectionPoolTrait>>>;

pub const USER_TABLE: &str = "core_user";
