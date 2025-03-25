use dirtybase_contract::{
    app::Context,
    auth::{AuthUser, ParseToken},
    prelude::{Credentials, Request, axum_extra},
};

use crate::StorageResolver;

pub async fn jwt_auth(req: Request) -> (Request, Result<Option<AuthUser>, anyhow::Error>) {
    let context = req.extensions().get::<Context>().unwrap();
    if let Some(header) = req.headers().get("authorization") {
        let bearer = axum_extra::headers::authorization::Bearer::decode(header);
        if let Some(cred) = bearer {
            let token = cred.token().to_string();
            if token.contains("|") {
                if let Ok(token) = ParseToken::try_from(token) {
                    let storage = StorageResolver::new(context.clone())
                        .get_provider()
                        .await
                        .unwrap();

                    return (req, storage.find_by_id(token.id()).await);
                }
            }
        }
    }

    (req, Ok(None))
}
