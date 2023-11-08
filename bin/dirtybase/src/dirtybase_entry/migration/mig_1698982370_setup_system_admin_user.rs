use dirtybase_contract::db::base::manager::Manager;
use dirtybase_contract::db::migration::Migration;

pub struct Mig1698982370setupsystemadminuser;

#[dirtybase_contract::async_trait]
impl Migration for Mig1698982370setupsystemadminuser {
    async fn up(&self, manager: &Manager) {
        println!("This is a test going up");
    }

    async fn down(&self, manager: &Manager) {
        println!("This is a test going down");
    }
}
