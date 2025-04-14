use dirtybase_contract::db_contract::migration::Migration;
use dirtybase_contract::db_contract::base::manager::Manager;

pub struct Mig1744604919CreateCacheTables;

#[dirtybase_contract::async_trait]
impl Migration for Mig1744604919CreateCacheTables {
  async fn up(&self, manager: &Manager)-> Result<(), anyhow::Error> {
     println!("going up: {}", "Mig1744604919CreateCacheTables");
     Ok(())
  }

  async fn down(&self, manager: &Manager) -> Result<(), anyhow::Error> {
     println!("going down: {}", "Mig1744604919CreateCacheTables");
     Ok(())
  }
}