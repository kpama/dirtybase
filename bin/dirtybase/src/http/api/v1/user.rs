use actix_web::Scope;

mod v1_open_user_login;

pub fn register_routes(scope: Scope) -> Scope {
    scope
}

pub fn register_public_routes(scope: Scope) -> Scope {
    scope.service(v1_open_user_login::user_login_handler)
}
