// pub use dirtybase_db::*;
// pub use dirtybase_db_macro as macros;
pub mod base;
pub mod config;
// pub mod entity;
pub mod connection_bus;
pub mod event;
pub mod field_values;
pub mod migration;
pub mod query_values;
pub mod relations;
pub mod table_entity;
pub mod types;

mod column_value_builder;

use std::collections::HashMap;

use base::{
    connection::ConnectionPoolTrait,
    schema::{ClientType, DatabaseKind},
};
pub use column_value_builder::*;
pub use table_entity::*;

pub const USER_TABLE: &str = "users";

pub type PoolManagerSet = HashMap<ClientType, Box<dyn ConnectionPoolTrait>>;

pub type DatabaseKindPoolCollection = HashMap<DatabaseKind, PoolManagerSet>;
