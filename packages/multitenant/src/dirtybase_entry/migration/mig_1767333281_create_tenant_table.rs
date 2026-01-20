use dirtybase_common::db::TableModel;
use dirtybase_contract::anyhow;
use dirtybase_contract::db_contract::base::manager::Manager;
use dirtybase_contract::db_contract::migration::Migration;
use dirtybase_contract::multitenant_contract::model::{Tenant, TenantStatus};

pub struct Mig1767333281CreateTenantTable;

#[dirtybase_contract::async_trait]
impl Migration for Mig1767333281CreateTenantTable {
    async fn up(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        _ = manager
            .create_table_schema(Tenant::table_name(), |bp| {
                bp.uuid_as_id(Some(Tenant::id_column()));
                bp.string(Tenant::col_name_for_name());
                bp.string(Tenant::col_name_for_token());
                bp.string(Tenant::col_name_for_status())
                    .set_default(TenantStatus::Pending);
                bp.sized_string(Tenant::col_name_for_domain(), 512)
                    .nullable();
                bp.timestamps();
                bp.soft_deletable();
                bp.unique_index(&[Tenant::col_name_for_name()]);
                bp.unique_index(&[Tenant::col_name_for_token()]);
                bp.unique_index(&[Tenant::col_name_for_domain()]);
            })
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &Manager) -> Result<(), anyhow::Error> {
        _ = manager.drop_table(Tenant::table_name()).await?;
        Ok(())
    }
}
