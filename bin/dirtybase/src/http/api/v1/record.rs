use actix_web::Scope;

mod v1_create_record;
mod v1_delete_record;
mod v1_get_a_record;
mod v1_get_all_records;
mod v1_update_record;

pub fn register_routes(scope: Scope) -> Scope {
    scope
        .service(v1_get_all_records::get_all_records_handler)
        .service(v1_get_a_record::get_a_record_handler)
        .service(v1_create_record::create_record_handler)
        .service(v1_update_record::update_record_handler)
        .service(v1_delete_record::delete_record_handler)
}
