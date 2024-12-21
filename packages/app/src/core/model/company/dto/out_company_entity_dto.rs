use dirtybase_db::types::UlidField;
use dirtybase_db_macro::DirtyTable;

use crate::core::model::company::CompanyEntity;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize, Clone, DirtyTable)]
pub struct OutCompanyEntityDto {
    id: UlidField,
    name: String,
    description: String,
}

impl From<CompanyEntity> for OutCompanyEntityDto {
    fn from(value: CompanyEntity) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description.unwrap_or_default(),
        }
    }
}
