use crate::app::DirtyBase;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct FakeUser {
    id: String,
    name: String,
    token: String,
}

impl FakeUser {
    fn mock(id: String, token: &str) -> Self {
        Self {
            name: format!("Fake User {}", &id),
            id,
            token: token.into(),
        }
    }
}

#[get("/_admin/users")]
async fn list_users_handler(_req: HttpRequest, app: web::Data<DirtyBase>) -> impl Responder {
    let mut users = Vec::<FakeUser>::new();

    for id in 0..=255 {
        let mut claim = HashMap::<String, String>::new();

        claim.insert("sub".into(), id.to_string());
        claim.insert("r".into(), "something".into());
        claim.insert("t".into(), "Default Tenant".into());
        claim.insert("a".into(), "Default App".into());

        if let Some(jwt) = app.sign_to_jwt(claim) {
            users.push(FakeUser::mock(id.to_string(), &jwt));
        }
    }

    HttpResponse::Ok().json(users)
}
