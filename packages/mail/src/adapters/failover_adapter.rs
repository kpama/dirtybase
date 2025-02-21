use crate::{AdapterTrait, email::Envelope};

pub struct FailoverAdapter;

#[async_trait::async_trait]
impl AdapterTrait for FailoverAdapter {
    fn name(&self) -> &str {
        "failover"
    }

    async fn send(&self, _envelope: Envelope) -> Result<bool, anyhow::Error> {
        unimplemented!()
    }
}
