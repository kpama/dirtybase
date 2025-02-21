use dirtybase_contract::db::base::manager::Manager;
use dirtybase_contract::db::migration::Migration;

pub struct Mig0000000000CreateUserTable;

#[dirtybase_contract::async_trait]
impl Migration for Mig0000000000CreateUserTable {
    async fn up(&self, manager: &Manager) {
        println!("going up: {}", "Mig0000000000CreateUserTable");
    }

    async fn down(&self, manager: &Manager) {
        println!("going down: {}", "Mig0000000000CreateUserTable");
    }
}
// 0000000000
