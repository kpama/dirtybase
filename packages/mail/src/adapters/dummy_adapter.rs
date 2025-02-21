use crate::{AdapterTrait, email::Envelope};

pub struct DummyAdapter;

#[async_trait::async_trait]
impl AdapterTrait for DummyAdapter {
    fn name(&self) -> &str {
        "dummy"
    }

    async fn send(&self, _envelope: Envelope) -> Result<bool, anyhow::Error> {
        log::debug!(target: "dummy email", "{:#?}", "dry test for now");

        Ok(true)
    }
}
