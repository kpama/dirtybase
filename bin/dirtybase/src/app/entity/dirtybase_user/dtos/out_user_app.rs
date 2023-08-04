use crate::app::entity::{
    app::AppEntity, company::dto::out_company_entity_dto::OutCompanyEntityDto,
    role::dtos::out_role_dto::OutRoleEntityDto,
};
use dirtybase_db::macros::DirtyTable;

#[derive(Debug, Clone, serde::Serialize, DirtyTable, Default)]
pub struct UserAppDto {
    pub id: String,
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
            id: value.id.unwrap_or_default(),
            name: value.name.unwrap_or_default(),
            description: value.description.unwrap_or_default(),
            is_system_app: value.is_system_app.unwrap_or_default(),
            company: value.company.into(),
            roles: Vec::new(),
        }
    }
}
