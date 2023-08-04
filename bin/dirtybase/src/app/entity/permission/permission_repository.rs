use super::permission_entity::PermissionEntity;
use crate::app::DirtyBase;
use dirtybase_db::base::manager::Manager;
use dirtybase_db_types::{field_values::FieldValue, types::IntoColumnAndValue, TableEntityTrait};

pub struct PermissionRepository {
    manager: Manager,
}

impl PermissionRepository {
    pub fn new(manager: Manager) -> Self {
        Self { manager }
    }

    pub fn manager(&self) -> &Manager {
        &self.manager
    }

    pub async fn by_name(
        &self,
        name: &str,
        with_trashed: bool,
    ) -> Result<PermissionEntity, anyhow::Error> {
        self.manager()
            .select_from_table(PermissionEntity::table_name(), |query| {
                query.select_all();
                query.eq(PermissionEntity::col_name_for_name(), name);

                if with_trashed {
                    query.with_trash();
                }
            })
            .fetch_one_to()
            .await
    }

    pub async fn by_id(
        &self,
        id: &str,
        with_trashed: bool,
    ) -> Result<PermissionEntity, anyhow::Error> {
        self.manager()
            .select_from_table(PermissionEntity::table_name(), |query| {
                query.select_all();
                query.eq(PermissionEntity::col_name_for_id(), id);

                if with_trashed {
                    query.with_trash();
                }
            })
            .fetch_one_to()
            .await
    }

    pub async fn all_by_company_id(
        &self,
        company_id: &str,
        with_trashed: bool,
    ) -> Result<Vec<PermissionEntity>, anyhow::Error> {
        self.manager()
            .select_from_table(PermissionEntity::table_name(), |query| {
                query
                    .select_all()
                    .eq(PermissionEntity::col_name_for_core_company_id(), company_id);

                if with_trashed {
                    query.with_trash();
                }
            })
            .fetch_all_to()
            .await
    }

    pub async fn create(
        &self,
        record: impl IntoColumnAndValue,
    ) -> Result<PermissionEntity, anyhow::Error> {
        let kv = record.into_column_value();
        let id: String =
            FieldValue::from_ref_option_into(kv.get(PermissionEntity::col_name_for_id()));

        self.manager()
            .insert(PermissionEntity::table_name(), kv)
            .await;

        self.by_name(&id, false).await
    }

    pub async fn update(
        &self,
        record: impl IntoColumnAndValue,
        id: &str,
    ) -> Result<PermissionEntity, anyhow::Error> {
        self.manager()
            .update(
                PermissionEntity::table_name(),
                record.into_column_value(),
                |query| {
                    query.eq(PermissionEntity::col_name_for_id(), id);
                },
            )
            .await;

        self.by_id(id, false).await
    }
}

#[busybody::async_trait]
impl busybody::Injectable for PermissionRepository {
    async fn inject(ci: &busybody::ServiceContainer) -> Self {
        let app: busybody::Service<DirtyBase> = ci.get().unwrap();

        Self::new(app.schema_manger())
    }
}
