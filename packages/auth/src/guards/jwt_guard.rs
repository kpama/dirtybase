use dirtybase_contract::{
    auth::{AuthUser, AuthUserStorageProvider},
    prelude::Request,
};

pub async fn authenticate(
    req: Request,
    user_provider: AuthUserStorageProvider,
) -> (Request, Result<Option<AuthUser>, anyhow::Error>) {
    (req, Ok(None))
}
