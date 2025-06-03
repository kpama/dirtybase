use std::{
    fmt::{Debug, Display},
    ops::Deref,
    sync::Arc,
};

mod actor;
mod actor_role;
mod permission;
mod role;
mod role_permission;

pub use actor::*;
pub use actor_role::*;
pub use permission::*;
pub use role::*;
pub use role_permission::*;
use serde::{Deserialize, Serialize};

use crate::db_contract::field_values::FieldValue;

pub enum PermissionRecordAction {
    SoftDelete,
    SoftRestore,
    PermDelete,
}

pub enum PermissionRelAction {
    Add,
    Remove,
}
