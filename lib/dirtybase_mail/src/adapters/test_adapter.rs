use async_trait::async_trait;

use crate::{adapter_manager::AdapterTrait, email::Envelope};

pub struct TestAdapter;

#[async_trait]
impl AdapterTrait for TestAdapter {
    async fn send(&self, envelope: Envelope) -> Result<bool, anyhow::Error> {
        Ok(true)
    }

    fn name(&self) -> &str {
        "test"
    }
}
