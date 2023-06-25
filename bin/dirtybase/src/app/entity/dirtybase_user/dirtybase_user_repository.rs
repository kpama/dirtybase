use dirtybase_db::base::manager::Manager;

use crate::app::DirtyBase;

pub struct DirtybaseUserRepository {
    manager: Manager,
}

impl DirtybaseUserRepository {
    pub fn new(manager: Manager) -> Self {
        Self { manager }
    }
}

#[busybody::async_trait]
impl busybody::Injectable for DirtybaseUserRepository {
    async fn inject(ci: &busybody::ServiceContainer) -> Self {
        let app = ci.get::<DirtyBase>().unwrap();

        Self::new(app.schema_manger())
    }
}
