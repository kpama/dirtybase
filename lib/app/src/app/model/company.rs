use dirtybase_contract::db::{
    base::{
        manager::Manager,
        table::{
            CREATED_AT_FIELD, CREATOR_FIELD, DELETED_AT_FIELD, EDITOR_FIELD, ID_FIELD,
            INTERNAL_ID_FIELD, UPDATED_AT_FIELD,
        },
    },
    entity::user::USER_TABLE,
};

mod company_entity;
mod company_repository;
mod company_service;

pub mod dto;

pub use company_entity::CompanyEntity;
pub use company_repository::CompanyRepository;
pub use company_service::CompanyService;

pub const COMPANY_TABLE: &str = "core_company";

// Table columns
pub const COMPANY_TABLE_NAME_FIELD: &str = "name";
pub const COMPANY_TABLE_DESCRIPTION_FIELD: &str = "description";
pub const COMPANY_TABLE_CORE_USER_ID_FIELD: &str = "core_user_id";
pub const COMPANY_TABLE_INTERNAL_ID_FIELD: &str = INTERNAL_ID_FIELD;
pub const COMPANY_TABLE_ID_FIELD: &str = ID_FIELD;
pub const COMPANY_TABLE_CREATOR_FIELD: &str = CREATOR_FIELD;
pub const COMPANY_TABLE_EDITOR_FIELD: &str = EDITOR_FIELD;
pub const COMPANY_TABLE_CREATED_AT_FIELD: &str = CREATED_AT_FIELD;
pub const COMPANY_TABLE_UPDATED_AT_FIELD: &str = UPDATED_AT_FIELD;
pub const COMPANY_TABLE_DELETED_AT_FIELD: &str = DELETED_AT_FIELD;

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
