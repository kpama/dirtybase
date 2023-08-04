use actix_web::Scope;

mod v1_user_public;
mod v1_user_secure;

pub fn register_routes(scope: Scope) -> Scope {
    scope.service(v1_user_secure::switch_app_handler)
}

pub fn register_public_routes(scope: Scope) -> Scope {
    scope.service(v1_user_public::user_login_handler)
}
