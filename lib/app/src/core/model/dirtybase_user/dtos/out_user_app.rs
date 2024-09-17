use crate::core::model::{
    app_entity::AppEntity, company::dto::out_company_entity_dto::OutCompanyEntityDto,
    role::dtos::out_role_dto::OutRoleEntityDto,
};
use dirtybase_db::types::UlidField;
use dirtybase_db_macro::DirtyTable;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, DirtyTable, Default)]
pub struct UserAppDto {
    pub id: UlidField,
    pub name: String,
    pub description: String,
    pub is_system_app: bool,
    pub company: OutCompanyEntityDto,
    #[dirty(skip_select)]
    pub roles: Vec<OutRoleEntityDto>,
}

impl From<AppEntity> for UserAppDto {
    fn from(value: AppEntity) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description.unwrap_or_default(),
            is_system_app: value.is_system_app,
            company: value.company.into(),
            roles: Vec::new(),
        }
    }
}
