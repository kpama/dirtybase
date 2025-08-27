use dirtybase_contract::{
    auth_contract::{AuthUserStorageProvider, StorageResolver},
    prelude::Context,
};

use crate::AuthExtension;

/// Resolves and return an instance of the storage provider
pub async fn get_auth_storage(
    ctx: Context,
    name: Option<&str>,
) -> Result<AuthUserStorageProvider, anyhow::Error> {
    let config = AuthExtension::config_from_ctx(&ctx).await?;
    if let Some(storage) = StorageResolver::from_context(ctx)
        .await
        .get_provider(name.unwrap_or_else(|| config.storage_as_str()))
        .await
    {
        Ok(storage)
    } else {
        Err(anyhow::anyhow!("Could not resolve auth storage"))
    }
}
