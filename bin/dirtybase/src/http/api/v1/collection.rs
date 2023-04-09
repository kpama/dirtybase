pub mod collections_records_controllers;

use actix_web::Scope;

pub fn register_routes(scope: Scope) -> Scope {
    scope
        .service(collections_records_controllers::get_all_records)
        .service(collections_records_controllers::get_a_record)
        .service(collections_records_controllers::create_record)
        .service(collections_records_controllers::update_record)
        .service(collections_records_controllers::delete_record)
}
