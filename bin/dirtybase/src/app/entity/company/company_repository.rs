#![allow(dead_code)]

use dirtybase_db::base::manager::Manager;

pub struct CompanyRepository {
    manager: Manager,
}

impl CompanyRepository {
    pub fn new(manager: Manager) -> Self {
        Self { manager }
    }

    pub fn manager(&self) -> &Manager {
        &self.manager
    }

    pub fn manager_mut(&mut self) -> &mut Manager {
        &mut self.manager
    }
}
