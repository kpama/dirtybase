use dirtybase_contract::{
    auth_contract::{AuthUserStorageProvider, StorageResolver},
    prelude::Context,
};

use crate::AuthConfig;

/// Resolves and return an instance of the storage provider
pub async fn get_auth_storage(
    ctx: Context,
    name: Option<&str>,
) -> Result<AuthUserStorageProvider, anyhow::Error> {
    let config = ctx.get_config::<AuthConfig>("auth").await?;
    if let Some(storage) = StorageResolver::from_context(ctx)
        .await
        .get_provider(if name.is_some() {
            name.unwrap()
        } else {
            config.storage_as_str()
        })
        .await
    {
        Ok(storage)
    } else {
        Err(anyhow::anyhow!("Could not resolve auth storage"))
    }
}
