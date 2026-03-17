use serde::{Deserialize, Serialize};

use crate::db::field_values::FieldValue;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnBlueprint {
    pub name: String,
    pub after: Option<String>,
    pub column_type: ColumnType,
    pub default: Option<ColumnDefault>,
    pub is_unique: bool,
    pub is_primary: bool,
    pub is_nullable: Option<bool>,
    pub relationship: Option<ForeignKey>,
    pub check: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForeignKey {
    table: String,
    column: String,
    cascade_delete: bool,
    cascade_update: bool,
}

impl ForeignKey {
    pub fn new(table: &str, column: &str, cascade_delete: bool, cascade_update: bool) -> Self {
        Self {
            table: table.to_owned(),
            column: column.to_owned(),
            cascade_delete,
            cascade_update,
        }
    }

    pub fn table(&self) -> String {
        self.table.clone()
    }

    pub fn column(&self) -> String {
        self.column.clone()
    }
    pub fn cascade_delete(&self) -> bool {
        self.cascade_delete
    }

    pub fn set_cascase_delete(&mut self, cascade_delete: bool) -> &mut Self {
        self.cascade_delete = cascade_delete;
        self
    }

    pub fn cascade_update(&self) -> bool {
        self.cascade_update
    }

    pub fn set_cascase_update(&mut self, cascade_update: bool) -> &mut Self {
        self.cascade_update = cascade_update;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColumnDefault {
    Custom(String),
    Boolean(bool),
    EmptyString,
    Zero,
    EmptyObject,
    EmptyArray,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColumnType {
    AutoIncrementId,
    Boolean,
    Char(usize),
    Datetime,
    Date,
    Timestamp,
    Float,
    Integer,
    Json,
    Binary,
    Enum(Vec<String>),
    Number,
    String(usize),
    Text,
    Uuid,
}

impl ColumnBlueprint {
    pub fn new(name: &str, column_type: ColumnType) -> Self {
        Self {
            name: name.to_string(),
            column_type,
            default: None,
            after: None,
            is_unique: false,
            is_primary: false,
            is_nullable: Some(false),
            relationship: None,
            check: None,
        }
    }

    pub fn set_type(&mut self, t: ColumnType) -> &mut Self {
        self.column_type = t;
        self
    }

    pub fn set_default<D: ToString>(&mut self, default: D) -> &mut Self {
        self.default = Some(ColumnDefault::Custom(default.to_string()));
        self
    }

    pub fn set_default_from<T: Into<FieldValue>>(&mut self, value: T) -> &mut Self {
        self.default = Some(ColumnDefault::Custom(value.into().to_string()));
        self
    }

    pub fn default_is_empty_string(&mut self) -> &mut Self {
        self.default = Some(ColumnDefault::EmptyString);
        self
    }

    pub fn default_is_true(&mut self) -> &mut Self {
        self.default = Some(ColumnDefault::Boolean(true));
        self
    }
    pub fn default_is_false(&mut self) -> &mut Self {
        self.default = Some(ColumnDefault::Boolean(false));
        self
    }

    pub fn default_is_zero(&mut self) -> &mut Self {
        self.default = Some(ColumnDefault::Zero);
        self
    }

    pub fn default_is_empty_object(&mut self) -> &mut Self {
        self.default = Some(ColumnDefault::EmptyObject);
        self
    }

    pub fn default_is_empty_array(&mut self) -> &mut Self {
        self.default = Some(ColumnDefault::EmptyArray);
        self
    }

    pub fn unset_default(&mut self) -> &mut Self {
        self.default = None;
        self
    }

    pub fn set_after(&mut self, after: &str) -> &mut Self {
        self.after = if after.is_empty() {
            None
        } else {
            Some(after.to_owned())
        };
        self
    }

    pub fn set_is_unique(&mut self, unique: bool) -> &mut Self {
        self.is_unique = unique;
        self
    }

    pub fn set_as_primary(&mut self) -> &mut Self {
        self.is_primary = true;
        self
    }

    pub fn set_check(&mut self, check: &str) -> &mut Self {
        self.check = Some(check.to_string());
        self
    }

    pub fn nullable(&mut self) -> &mut Self {
        self.is_nullable = Some(true);
        self
    }

    pub fn set_is_nullable(&mut self, nullable: bool) -> &mut Self {
        self.is_nullable = Some(nullable);
        self
    }

    pub fn references(
        &mut self,
        table: &str,
        column: &str,
        cascade_delete: bool,
        cascade_update: bool,
    ) -> &mut Self {
        self.relationship = Some(ForeignKey::new(
            table,
            column,
            cascade_delete,
            cascade_update,
        ));
        self
    }

    pub fn relationship(&mut self) -> Option<&mut ForeignKey> {
        self.relationship.as_mut()
    }
    pub fn relationship_fn<F>(&mut self, callback: F) -> &mut Self
    where
        F: FnOnce(&mut ForeignKey),
    {
        if let Some(rel) = self.relationship.as_mut() {
            callback(rel);
        }
        self
    }

    pub fn references_with_cascade_delete(&mut self, table: &str, column: &str) -> &mut Self {
        self.references(table, column, true, false)
    }
    pub fn references_with_cascade_update(&mut self, table: &str, column: &str) -> &mut Self {
        self.references(table, column, false, true)
    }

    pub fn reference_cascade_all(&mut self, table: &str, column: &str) -> &mut Self {
        self.references(table, column, true, true)
    }
}
