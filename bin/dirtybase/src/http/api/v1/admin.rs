use actix_web::Scope;

mod v1_create_user;
mod v1_list_users;

pub fn register_routers(scope: Scope) -> Scope {
    scope
        .service(v1_list_users::list_users_handler)
        .service(v1_create_user::create_user_handler)
}
