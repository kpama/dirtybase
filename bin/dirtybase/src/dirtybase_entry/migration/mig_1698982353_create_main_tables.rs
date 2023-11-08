use dirtybase_contract::db::base::manager::Manager;
use dirtybase_contract::db::migration::Migration;

use crate::app::{
    setup_database::{create_default_tables, drop_default_tables},
    setup_defaults::setup_default_entities,
};

pub struct Mig1698982353createmaintables;

#[dirtybase_contract::async_trait]
impl Migration for Mig1698982353createmaintables {
    async fn up(&self, manager: &Manager) {
        create_default_tables(manager).await;
        setup_default_entities().await;
    }

    async fn down(&self, manager: &Manager) {
        drop_default_tables(manager).await;
    }
}
