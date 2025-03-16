#![allow(unused)]

use super::base::manager::Manager;

#[async_trait::async_trait]
pub trait Migration: Send + Sync {
    async fn up(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        // Migrate up
        Ok(())
    }

    async fn down(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        // Migrate down
        Ok(())
    }

    fn id(&self) -> String {
        std::any::type_name::<Self>()
            .split("::")
            .last()
            .unwrap()
            .to_lowercase()
    }
}
