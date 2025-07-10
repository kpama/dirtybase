use self::permission_entity::PermissionEntity;
use super::company::CompanyEntity;
use dirtybase_db::base::manager::Manager;
use dirtybase_db::TableModel;

mod permission_validator;

pub use permission_validator::PermissionValidator;

pub mod permission_entity;
pub mod permission_repository;
pub mod permission_service;

pub async fn setup_permission_table(manager: &Manager) {
    // TODO: Make each role unique based on it's name and company fields
    manager
        .create_table_schema(PermissionEntity::table_name(), |table| {
            // internal_id
            // id
            table.id_set();

            // company_id
            table.ulid_fk(CompanyEntity::table_name(), true);

            // name
            table.string(PermissionEntity::col_name_for_name());

            // label
            table.string(PermissionEntity::col_name_for_label());

            // description
            table
                .sized_string(PermissionEntity::col_name_for_description(), 512)
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
