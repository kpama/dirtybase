use serde::{Deserialize, Serialize};

mod actor;
mod permission;
mod role;

pub use actor::*;
pub use permission::*;
pub use role::*;

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
