use dirtybase_contract::db_contract::types::{
    FromColumnAndValue, IntegerField, InternalIdField, OptionalDateTimeField, StringField,
};

pub(crate) const TABLE_NAME: &str = "migrations";
pub(crate) const NAME_COLUMN: &str = "name";
pub(crate) const BATCH_COLUMN: &str = "batch";
pub(crate) const CREATED_AT_COLUMN: &str = "created_at";

#[derive(Debug, Default, Clone)]
pub struct MigrationEntity {
    pub(crate) _id: InternalIdField,
    pub(crate) name: StringField,
    pub(crate) batch: IntegerField,
    pub(crate) _created_at: OptionalDateTimeField,
}

impl FromColumnAndValue for MigrationEntity {
    fn from_column_value(cv: dirtybase_contract::db_contract::types::ColumnAndValue) -> Self {
        Self {
            _id: cv.get("id").unwrap().into(),
            name: cv.get(NAME_COLUMN).unwrap().into(),
            batch: cv.get(BATCH_COLUMN).unwrap().into(),
            _created_at: cv.get(CREATED_AT_COLUMN).unwrap().into(),
        }
    }
}
