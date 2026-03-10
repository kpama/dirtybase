use dirtybase_contract::{
    auth_contract::{AuthUserStorageProvider, StorageResolver, storage2::PermStorageProvider},
    prelude::Context,
};

use crate::AuthExtension;

/// Resolves and return an instance of the storage provider
pub async fn get_auth_storage(ctx: Context) -> Result<PermStorageProvider, anyhow::Error> {
    ctx.get().await
}
