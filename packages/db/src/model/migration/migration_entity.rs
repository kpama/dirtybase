use anyhow::anyhow;
use dirtybase_contract::db_contract::types::{
    FromColumnAndValue, IntegerField, InternalIdField, OptionalDateTimeField, StringField,
};

pub(crate) const TABLE_NAME: &str = "migrations";
pub(crate) const NAME_COLUMN: &str = "name";
pub(crate) const BATCH_COLUMN: &str = "batch";
pub(crate) const CREATED_AT_COLUMN: &str = "created_at";

#[derive(Debug, Default, Clone)]
pub struct MigrationEntity {
    pub(crate) id: InternalIdField,
    pub(crate) name: StringField,
    pub(crate) batch: IntegerField,
    pub(crate) created_at: OptionalDateTimeField,
}

impl FromColumnAndValue for MigrationEntity {
    fn from_column_value(
        mut cv: dirtybase_contract::db_contract::types::ColumnAndValue,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            id: cv
                .remove("id")
                .map(InternalIdField::from)
                .ok_or(anyhow!("migration entity id field is missing"))?
                .into(),
            name: cv
                .remove(NAME_COLUMN)
                .map(StringField::from)
                .ok_or(anyhow!("migration entity name field is missing"))?
                .into(),
            batch: cv
                .get(BATCH_COLUMN)
                .map(IntegerField::from)
                .ok_or(anyhow!("migration entity batch field is missing"))?
                .into(),
            created_at: cv
                .get(CREATED_AT_COLUMN)
                .map(OptionalDateTimeField::from)
                .ok_or(anyhow!("migration entity created_at field is missing"))?
                .into(),
        })
    }
}
