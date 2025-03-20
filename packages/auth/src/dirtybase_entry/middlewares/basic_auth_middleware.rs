use dirtybase_contract::{
    app::Context,
    http::prelude::*,
    user::{UserProviderService, model::UserRepositoryTrait},
};

pub async fn handle_basic_auth_middleware(req: Request, next: Next) -> impl IntoResponse {
    log::debug!(">>>> Basic auth ran <<<<<");
    let context = req.extensions().get::<Context>().unwrap();
    let user_provider = context.get::<UserProviderService>().await.unwrap();

    let mut auth_passed = req.headers().get("authorization").is_some();
    if let Some(header) = req.headers().get("authorization") {
        match axum_extra::headers::authorization::Basic::decode(header) {
            Some(cred) => {
                let username = cred.username();
                let password = cred.password();

                // FIXME: This should be checking the username and password from the user provider
                if username.is_empty() || password.is_empty() {
                    auth_passed = false;
                }

                // let user = user_provider.get_user_by_username(&username);
                // let is_valid =  auth_provider.validate(&user.password_hash(), password);
                // let is_valid =  auth_provider.user_by_username(username, password);

                // check the username and password
                let hash_password = user_provider.find_by_username(username, false).await;
                println!("user with id ({}): {}", &username, &hash_password.is_ok());

                // if password != hash_password {
                //     auth_passed = false;
                // } else if let Some(context) = req.extensions().get::<Context>() {
                //     context.set(UserContext::default()).await; //FIXME: source the real user using the user provider
                // }
            }
            None => auth_passed = false,
        }
    }

    if !auth_passed {
        let mut response = Response::new(Body::from("Unauthorized"));
        *response.status_mut() = StatusCode::UNAUTHORIZED;
        return response;
    }

    if let Some(header) = req.headers().get("authorization") {
        dbg!("auth header after change: {:?}", header);
    }

    // success, call the next middleware
    next.run(req).await
}
