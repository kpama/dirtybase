use dirtybase_contract::db_contract::types::{
    FromColumnAndValue, IntegerField, InternalIdField, OptionalDateTimeField, StringField,
};

pub(crate) const TABLE_NAME: &str = "migrations";
pub(crate) const NAME_COLUMN: &str = "name";
pub(crate) const BATCH_COLUMN: &str = "batch";
pub(crate) const CREATED_AT_COLUMN: &str = "created_at";

#[derive(Debug, Default, Clone)]
// #[dirty(table = "core_migration", id = "id")]
pub struct MigrationEntity {
    pub(crate) id: InternalIdField,
    pub(crate) name: StringField,
    pub(crate) batch: IntegerField,
    pub(crate) created_at: OptionalDateTimeField,
}

impl FromColumnAndValue for MigrationEntity {
    fn from_column_value(cv: dirtybase_contract::db_contract::types::ColumnAndValue) -> Self {
        Self {
            id: cv.get("id").unwrap().into(),
            name: cv.get(NAME_COLUMN).unwrap().into(),
            batch: cv.get(BATCH_COLUMN).unwrap().into(),
            created_at: cv.get(CREATED_AT_COLUMN).unwrap().into(),
        }
    }
}
