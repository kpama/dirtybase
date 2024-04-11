use crate::{email::Envelope, AdapterTrait};

pub struct DummyAdapter;

#[async_trait::async_trait]
impl AdapterTrait for DummyAdapter {
    fn name(&self) -> &str {
        "dummy"
    }

    async fn send(&self, envelope: Envelope) -> Result<bool, anyhow::Error> {
        log::debug!(target: "dummy email", "{:#?}", "dry test for now");

        Ok(true)
    }
}
