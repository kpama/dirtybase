use std::fmt::Debug;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionRecordAction {
    SoftDelete,
    SoftRestore,
    PermDelete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionRelAction {
    Add,
    Remove,
}
