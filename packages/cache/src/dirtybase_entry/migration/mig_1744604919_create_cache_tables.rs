use dirtybase_contract::db_contract::base::manager::Manager;
use dirtybase_contract::db_contract::migration::Migration;

pub struct Mig1744604919CreateCacheTables;

#[dirtybase_contract::async_trait]
impl Migration for Mig1744604919CreateCacheTables {
    async fn up(&self, _manager: &Manager) -> Result<(), anyhow::Error> {
        println!("going up: Mig1744604919CreateCacheTables");
        Ok(())
    }

    async fn down(&self, _manager: &Manager) -> Result<(), anyhow::Error> {
        println!("going down: Mig1744604919CreateCacheTables");
        Ok(())
    }
}
