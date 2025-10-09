pub mod base;
pub mod event;
// pub mod field_values;
pub mod migration;
// pub mod query_column;
// pub mod query_values;
// pub mod table_model;
pub mod types;

// mod column_value_builder;
mod seeder_registerer;

use std::collections::HashMap;

use base::{
    connection::ConnectionPoolTrait,
    schema::{ClientType, DatabaseKind},
};
// pub use column_value_builder::*;
// pub use dirtybase_shared_type::db::field_values;
// pub use dirtybase_shared_type::db::query_column;
// pub use dirtybase_shared_type::db::query_values;
pub use dirtybase_shared_type::db::table_model::*;
pub use dirtybase_shared_type::db::*;

pub use seeder_registerer::*;
// pub use table_model::*;

pub type PoolManagerSet = HashMap<ClientType, Box<dyn ConnectionPoolTrait>>;

pub type DatabaseKindPoolCollection = HashMap<DatabaseKind, PoolManagerSet>;
