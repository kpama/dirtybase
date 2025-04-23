use std::{collections::HashMap, sync::OnceLock};
use tokio::sync::RwLock;

use async_trait::async_trait;

use crate::email::Envelope;

pub(crate) static REGISTERED_ADAPTERS: OnceLock<RwLock<HashMap<String, Box<dyn AdapterTrait>>>> =
    OnceLock::new();

#[async_trait]
pub trait AdapterTrait: Send + Sync + 'static {
    async fn send(&self, envelope: Envelope) -> Result<bool, anyhow::Error>;

    fn name(&self) -> &str;

    async fn register(self)
    where
        Self: Sized,
    {
        register_adapter(Box::new(self)).await;
    }
}

pub async fn register_adapter(adapter: Box<dyn AdapterTrait>) {
    let lock = REGISTERED_ADAPTERS.get_or_init(|| RwLock::new(HashMap::new()));
    let mut adapters = lock.write().await;

    adapters.insert(adapter.name().to_string(), adapter);
}
