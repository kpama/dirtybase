use dirtybase_contract::db::{base::manager::Manager, entity::user::UserEntity};

mod company_entity;
mod company_repository;
mod company_service;

pub mod dto;

pub use company_entity::CompanyEntity;
pub use company_repository::CompanyRepository;
pub use company_service::CompanyService;
use dirtybase_db::TableEntityTrait;

pub async fn setup_company_table(manager: &Manager) {
    manager
        .create_table_schema(CompanyEntity::table_name(), |table| {
            // internal_id
            // id
            table.id_set();
            // name
            table.string(CompanyEntity::col_name_for_name());
            // owner
            table
                .ulid_fk(UserEntity::table_name(), false)
                .set_is_nullable(true);
            // description
            table
                .sized_string(CompanyEntity::col_name_for_description(), 512)
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
