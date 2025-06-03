pub mod base;
pub mod event;
pub mod field_values;
pub mod migration;
pub mod query_values;
pub mod relations;
pub mod table_entity;
pub mod types;

mod column_value_builder;
mod seeder_registerer;

use std::collections::HashMap;

use base::{
    connection::ConnectionPoolTrait,
    schema::{ClientType, DatabaseKind},
};
pub use column_value_builder::*;
pub use seeder_registerer::*;
pub use table_entity::*;

pub type PoolManagerSet = HashMap<ClientType, Box<dyn ConnectionPoolTrait>>;

pub type DatabaseKindPoolCollection = HashMap<DatabaseKind, PoolManagerSet>;
