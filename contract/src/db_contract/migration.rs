#![allow(unused)]

use crate::prelude::Context;

use super::base::manager::Manager;

#[async_trait::async_trait]
pub trait Migration: Send + Sync {
    /// Set up things before running the migration
    async fn setup(&self, context: &Context) -> Result<(), anyhow::Error> {
        Ok(())
    }

    /// Check if the migration should be run
    async fn should_run(&self, context: &Context) -> bool {
        true
    }

    /// Migrate up aka apply the migration
    async fn up(&self, manager: &Manager, context: &Context) -> Result<(), anyhow::Error> {
        Ok(())
    }

    /// Migrate down aka revert the migration
    async fn down(&self, manager: &Manager, context: &Context) -> Result<(), anyhow::Error> {
        Ok(())
    }

    /// Get the migration unique id
    fn id(&self) -> String {
        std::any::type_name::<Self>()
            .split("::")
            .last()
            .unwrap()
            .to_lowercase()
    }
}
