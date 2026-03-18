use dirtybase_contract::{auth_contract::storage::PermStorageProvider, prelude::Context};

/// Resolves and return an instance of the storage provider
pub async fn get_auth_storage(ctx: Context) -> Result<PermStorageProvider, anyhow::Error> {
    ctx.get().await
}
