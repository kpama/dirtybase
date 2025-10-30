pub mod base;
pub mod column_value_builder;
pub mod event;
pub mod field_values;
pub mod query_column;
pub mod query_values;
pub mod table_model;
pub mod types;

use std::collections::HashMap;

pub use column_value_builder::*;
pub use table_model::*;

use crate::db::base::{
    connection::ConnectionPoolTrait,
    schema::{ClientType, DatabaseKind},
};

pub type PoolManagerSet = HashMap<ClientType, Box<dyn ConnectionPoolTrait>>;

pub type DatabaseKindPoolCollection = HashMap<DatabaseKind, PoolManagerSet>;
