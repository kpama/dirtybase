use dirtybase_db::macros::DirtyTable;
use dirtybase_db_types::types::{StringField, UlidField};

use crate::app::entity::company::CompanyEntity;

#[derive(Debug, Default, serde::Serialize, Clone, DirtyTable)]
pub struct OutCompanyEntityDto {
    id: String,
    name: String,
    description: String,
}

impl From<CompanyEntity> for OutCompanyEntityDto {
    fn from(value: CompanyEntity) -> Self {
        Self {
            id: value.id.unwrap_or_default(),
            name: value.name.unwrap_or_default(),
            description: value.description.unwrap_or_default(),
        }
    }
}
