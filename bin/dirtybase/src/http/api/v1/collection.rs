use actix_web::Scope;

mod v1_create_a_collection;
mod v1_delete_a_collection;
mod v1_get_a_collection;
mod v1_get_all_collections;
mod v1_update_a_collection;

pub fn register_routes(scope: Scope) -> Scope {
    scope
        .service(v1_get_all_collections::get_all_collections_handler)
        .service(v1_get_a_collection::get_a_collection_handler)
        .service(v1_create_a_collection::create_a_collection_handler)
        .service(v1_update_a_collection::update_a_collection_handler)
        .service(v1_delete_a_collection::delete_a_collection_handler)
}
