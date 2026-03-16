#![allow(dead_code)]

use std::collections::HashSet;

use dirtybase_db::{
    base::manager::Manager, connector::sqlite::make_sqlite_in_memory_manager, types::ArcUuid7,
};
use dirtybase_db_macro::DirtyTable;

#[tokio::main]
async fn main() {
    let _manager = setup_db().await;
}

async fn setup_db() -> Manager {
    let manager = make_sqlite_in_memory_manager().await;
    create_tables(&manager).await;
    manager
}

async fn create_tables(_manager: &Manager) {
    //
}

#[derive(Debug, Default, Clone, DirtyTable)]
struct TopUser {
    id: Option<ArcUuid7>,
    list: HashSet<i32>,
}
