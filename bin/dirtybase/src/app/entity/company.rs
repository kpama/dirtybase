use dirtybase_db::{base::manager::Manager, entity::user::USER_TABLE};

mod company_entity;
mod company_repository;
mod company_service;

pub use company_entity::CompanyEntity;
pub use company_repository::CompanyRepository;
pub use company_service::CompanyService;

pub const COMPANY_TABLE: &str = "core_company";
pub const COMPANY_TABLE_NAME_FIELD: &str = "name";
pub const COMPANY_TABLE_DESCRIPTION_FIELD: &str = "description";

pub async fn setup_company_table(manager: &Manager) {
    manager
        .create_table_schema(COMPANY_TABLE, |table| {
            // internal_id
            // id
            table.id_set();
            // name
            table.string(COMPANY_TABLE_NAME_FIELD);
            // owner
            table.ulid_fk(USER_TABLE, false).set_is_nullable(true);
            // description
            table
                .sized_string(COMPANY_TABLE_DESCRIPTION_FIELD, 512)
                .set_is_nullable(true);
            // blame
            table.blame();
            // timestamp
            table.timestamps();
            // soft delete
            table.soft_deletable();
        })
        .await;
}
