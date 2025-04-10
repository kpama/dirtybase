use crate::db_contract::field_values::FieldValue;

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct ForeignKey {
    table: String,
    column: String,
    cascade_delete: bool,
}

impl ForeignKey {
    pub fn new(table: &str, column: &str, cascade_delete: bool) -> Self {
        Self {
            table: table.to_owned(),
            column: column.to_owned(),
            cascade_delete,
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnDefault {
    Custom(String),
    EmptyString,
    CreatedAt,
    UpdatedAt,
    Zero,
    EmptyObject,
    EmptyArray,
    Uuid,
    Ulid,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnType {
    AutoIncrementId,
    Boolean,
    Char(usize),
    Datetime,
    Timestamp,
    Float,
    Integer,
    Json,
    // TODO: Add jsonb ? Jsonb,
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

    pub fn default_is_created_at(&mut self) -> &mut Self {
        self.default = Some(ColumnDefault::CreatedAt);
        self
    }

    pub fn default_is_uuid(&mut self) -> &mut Self {
        self.default = Some(ColumnDefault::Uuid);
        self
    }

    pub fn default_is_ulid(&mut self) -> &mut Self {
        self.default = Some(ColumnDefault::Ulid);
        self
    }

    pub fn default_is_updated_at(&mut self) -> &mut Self {
        self.default = Some(ColumnDefault::UpdatedAt);
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

    pub fn set_is_nullable(&mut self, nullable: bool) -> &mut Self {
        self.is_nullable = Some(nullable);
        self
    }

    pub fn references(&mut self, table: &str, column: &str, cascade_delete: bool) -> &mut Self {
        self.relationship = Some(ForeignKey::new(table, column, cascade_delete));
        self
    }
    pub fn references_with_cascade_delete(&mut self, table: &str, column: &str) -> &mut Self {
        self.references(table, column, true)
    }

    pub fn references_without_cascade_delete(&mut self, table: &str, column: &str) -> &mut Self {
        self.references(table, column, false)
    }
}
