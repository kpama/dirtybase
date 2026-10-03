use std::sync::Arc;

use dirtybase_db::{
    base::column::ColumnType,
    field_values::FieldValue,
    types::{ArcUuid7, UlidField},
};

use crate::error::{Error, ErrorList, Result};

/// The type of an attribute's value.
///
/// Each variant names a dirtybase field type, and knows both how to represent
/// that type at runtime as a [`FieldValue`] and how to declare it to the
/// database as a [`ColumnType`]. The `field_type_label` on each variant returns
/// the dirtybase field alias so introspection can report the concrete Rust type
/// a record should use.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttributeType {
    /// `dirtybase_common::db::types::StringField` (`Arc<String>`), varchar.
    String,
    /// `StringField` stored in an unbounded text column.
    Text,
    /// `dirtybase_common::db::types::IntegerField` (`i64`).
    Integer,
    /// `dirtybase_common::db::types::NumberField` (`f64`).
    Float,
    /// `dirtybase_common::db::types::BooleanField`.
    Boolean,
    /// `dirtybase_common::db::types::ArcUuid7`.
    Uuid,
    /// `dirtybase_common::db::types::UlidField`.
    Ulid,
    /// `dirtybase_common::db::types::TimestampField` (`DateTime<Utc>`).
    Timestamp,
    /// `dirtybase_common::db::types::DateField` (`NaiveDate`).
    Date,
    /// `dirtybase_common::db::types::JsonField` (`serde_json::Map`).
    Json,
    /// `serde_json::Value`, stored as json. For attributes with no fixed shape.
    JsonValue,
    /// `Vec<u8>`.
    Binary,
    /// A closed set of strings, declared to the database as an enum.
    Enum(Vec<String>),
}

impl AttributeType {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Self::Integer | Self::Float)
    }

    pub fn is_textual(&self) -> bool {
        matches!(self, Self::String | Self::Text | Self::Enum(_))
    }

    /// The dirtybase field type a record struct should use for this attribute.
    pub fn field_type_label(&self) -> String {
        match self {
            Self::String => "StringField (Arc<String>)".to_string(),
            Self::Text => "StringField (Arc<String>)".to_string(),
            Self::Integer => "IntegerField (i64)".to_string(),
            Self::Float => "NumberField (f64)".to_string(),
            Self::Boolean => "BooleanField (bool)".to_string(),
            Self::Uuid => "ArcUuid7".to_string(),
            Self::Ulid => "UlidField".to_string(),
            Self::Timestamp => "TimestampField (DateTime<Utc>)".to_string(),
            Self::Date => "DateField (NaiveDate)".to_string(),
            Self::Json => "JsonField (serde_json::Map<String, Value>)".to_string(),
            Self::JsonValue => "JsonValueField (serde_json::Value)".to_string(),
            Self::Binary => "Vec<u8>".to_string(),
            Self::Enum(options) => format!("one of [{}]", options.join(", ")),
        }
    }

    /// The column type to declare in the table blueprint.
    pub fn to_column_type(&self) -> ColumnType {
        match self {
            Self::String => ColumnType::String(255),
            Self::Text => ColumnType::Text,
            Self::Integer => ColumnType::Integer,
            Self::Float => ColumnType::Float,
            Self::Boolean => ColumnType::Boolean,
            Self::Uuid => ColumnType::Uuid,
            Self::Ulid => ColumnType::Char(26),
            Self::Timestamp => ColumnType::Timestamp,
            Self::Date => ColumnType::Date,
            Self::Json | Self::JsonValue => ColumnType::Json,
            Self::Binary => ColumnType::Binary,
            Self::Enum(options) => ColumnType::Enum(options.clone()),
        }
    }

    /// Normalise a value into the [`FieldValue`] variant this type expects.
    ///
    /// This routes through dirtybase's own `From<FieldValue>` conversions, so
    /// `"5"` becomes `IntegerField(5)` for an integer attribute and any
    /// mismatched variant yields the type's zero value rather than panicking.
    pub fn coerce(&self, value: FieldValue) -> FieldValue {
        match self {
            Self::String | Self::Text => {
                if matches!(
                    value,
                    FieldValue::String(_) | FieldValue::Null | FieldValue::NotSet
                ) {
                    value
                } else {
                    FieldValue::String(value.to_string())
                }
            }
            Self::Integer => match value {
                FieldValue::I64(v) => FieldValue::I64(v),
                FieldValue::I32(v) => FieldValue::I64(v as i64),
                FieldValue::I16(v) => FieldValue::I64(v as i64),
                FieldValue::I8(v) => FieldValue::I64(v as i64),
                FieldValue::U64(v) => FieldValue::I64(v as i64),
                FieldValue::U32(v) => FieldValue::I64(v as i64),
                FieldValue::F64(v) => FieldValue::I64(v as i64),
                FieldValue::Boolean(v) => FieldValue::I64(v as i64),
                FieldValue::String(ref s) => match s.parse::<i64>() {
                    Ok(v) => FieldValue::I64(v),
                    Err(_) => FieldValue::I64(0),
                },
                _ => FieldValue::I64(0),
            },
            Self::Float => match value {
                FieldValue::F64(v) => FieldValue::F64(v),
                FieldValue::I64(v) => FieldValue::F64(v as f64),
                FieldValue::I32(v) => FieldValue::F64(v as f64),
                FieldValue::U64(v) => FieldValue::F64(v as f64),
                FieldValue::U32(v) => FieldValue::F64(v as f64),
                FieldValue::String(ref s) => match s.parse::<f64>() {
                    Ok(v) => FieldValue::F64(v),
                    Err(_) => FieldValue::F64(0.0),
                },
                _ => FieldValue::F64(0.0),
            },
            Self::Boolean => match value {
                FieldValue::Boolean(v) => FieldValue::Boolean(v),
                FieldValue::I64(v) => FieldValue::Boolean(v != 0),
                FieldValue::F64(v) => FieldValue::Boolean(v != 0.0),
                FieldValue::String(ref s) => {
                    FieldValue::Boolean(matches!(s.to_lowercase().as_str(), "true" | "1" | "yes"))
                }
                _ => FieldValue::Boolean(false),
            },
            Self::Uuid => match value {
                FieldValue::Uuid(v) => FieldValue::Uuid(v),
                FieldValue::String(ref s) => {
                    // The canonical hyphenated form, which is what dirtybase
                    // hands out everywhere. `ArcUuid7::try_from` parses through
                    // `Uuid::parse_str`, which needs the hyphens — stripping
                    // them here would make every coercion fail and fall back to
                    // a fresh id. Keeping them means a caller's value survives
                    // a round trip.
                    match ArcUuid7::try_from(s.as_str()) {
                        Ok(v) => v.into(),
                        Err(_) => ArcUuid7::default().into(),
                    }
                }
                _ => ArcUuid7::default().into(),
            },
            Self::Ulid => match value {
                FieldValue::String(ref s) => FieldValue::String(s.clone()),
                _ => UlidField::default().into(),
            },
            Self::Timestamp => match value {
                // dirtybase's read path always yields `DateTime`, so that is the
                // one form soot keeps internally. Accepting `Timestamp` on the way
                // in means a value does not change shape across a round trip
                // through the database.
                FieldValue::DateTime(v) | FieldValue::Timestamp(v) => FieldValue::DateTime(v),
                FieldValue::I64(v) => FieldValue::DateTime(
                    chrono::DateTime::from_timestamp(v, 0).unwrap_or_else(chrono::Utc::now),
                ),
                FieldValue::String(ref s) => match chrono::DateTime::parse_from_rfc3339(s) {
                    Ok(v) => FieldValue::DateTime(v.with_timezone(&chrono::Utc)),
                    Err(_) => FieldValue::DateTime(chrono::Utc::now()),
                },
                _ => FieldValue::DateTime(chrono::Utc::now()),
            },
            Self::Date => match value {
                FieldValue::Date(v) => FieldValue::Date(v),
                FieldValue::String(ref s) => match chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                {
                    Ok(v) => FieldValue::Date(v),
                    Err(_) => FieldValue::Date(chrono::Utc::now().date_naive()),
                },
                _ => FieldValue::Date(chrono::Utc::now().date_naive()),
            },
            Self::Json => match value {
                FieldValue::Object(v) => FieldValue::Object(v),
                FieldValue::Null | FieldValue::NotSet => FieldValue::Object(Default::default()),
                other => match FieldValue::from(serde_json::Value::from(other)) {
                    FieldValue::Object(v) => FieldValue::Object(v),
                    _ => FieldValue::Object(Default::default()),
                },
            },
            Self::JsonValue => match value {
                FieldValue::Null | FieldValue::NotSet => FieldValue::Object(Default::default()),
                other => FieldValue::from(serde_json::Value::from(other)),
            },
            Self::Binary => match value {
                FieldValue::Binary(v) => FieldValue::Binary(v),
                _ => FieldValue::Binary(Vec::new()),
            },
            Self::Enum(_) => match value {
                FieldValue::String(v) => FieldValue::String(v),
                other => FieldValue::String(other.to_string()),
            },
        }
    }

    /// The value a freshly created record starts with, when the attribute
    /// declares no explicit default and the type has a natural one.
    pub fn implicit_default(&self) -> Option<FieldValue> {
        match self {
            Self::Uuid => Some(ArcUuid7::default().into()),
            Self::Ulid => Some(UlidField::default().into()),
            Self::Boolean => Some(FieldValue::Boolean(false)),
            Self::Integer => Some(FieldValue::I64(0)),
            Self::Float => Some(FieldValue::F64(0.0)),
            Self::String | Self::Text => Some(FieldValue::String(String::new())),
            Self::Json => Some(FieldValue::Object(Default::default())),
            Self::Binary => Some(FieldValue::Binary(Vec::new())),
            _ => None,
        }
    }
}

/// The rules an attribute's value must satisfy.
///
/// Ash nests these under `constraints`; keeping them in their own struct lets
/// validations read them generically without knowing which attributes use them.
#[derive(Debug, Clone, PartialEq)]
pub struct Constraints {
    pub allow_nil: bool,
    pub unique: bool,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    /// Closed value set. Enforced in addition to any `Enum` column type.
    pub values: Option<Vec<FieldValue>>,
    pub format: Option<String>,
}

/// A fresh set of constraints allows nil, so an attribute is optional until
/// [`Constraints::required`] says otherwise.
///
/// This matches Ash, where `allow_nil?` defaults to true and strictness is
/// opt-in. The opposite default is a trap: it makes the common case
/// (`Attribute::string("nickname")`) a declaration that rejects nil, and forces
/// every constructor that legitimately has no value — an enum, a json blob, a
/// free-text body — to call `optional()` to escape it.
impl Default for Constraints {
    fn default() -> Self {
        Self {
            allow_nil: true,
            unique: false,
            min: None,
            max: None,
            min_length: None,
            max_length: None,
            pattern: None,
            values: None,
            format: None,
        }
    }
}

impl Constraints {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reject a nil value. In Ash this is `allow_nil? false`, the default for
    /// every attribute that is not explicitly optional.
    pub fn required(mut self) -> Self {
        self.allow_nil = false;
        self
    }

    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    pub fn optional(mut self) -> Self {
        self.allow_nil = true;
        self
    }

    pub fn min(mut self, value: f64) -> Self {
        self.min = Some(value);
        self
    }

    pub fn max(mut self, value: f64) -> Self {
        self.max = Some(value);
        self
    }

    pub fn min_length(mut self, value: usize) -> Self {
        self.min_length = Some(value);
        self
    }

    pub fn max_length(mut self, value: usize) -> Self {
        self.max_length = Some(value);
        self
    }

    pub fn pattern(mut self, pattern: &str) -> Self {
        self.pattern = Some(pattern.to_string());
        self
    }

    pub fn values(mut self, values: Vec<FieldValue>) -> Self {
        self.values = Some(values);
        self
    }

    pub fn format(mut self, format: &str) -> Self {
        self.format = Some(format.to_string());
        self
    }

    /// Check a value against every constraint, collecting all violations.
    pub fn check(&self, name: &str, value: &FieldValue) -> Result<()> {
        let mut errors = ErrorList::new();

        if is_nil(value) {
            if !self.allow_nil {
                errors.push(Error::required(name, "is required"));
            }
            return errors.into_result();
        }

        if let Some(max_length) = self.max_length {
            if let Some(length) = value_length(value)
                && length > max_length
            {
                errors.push(
                    Error::invalid(name, "is too long")
                        .with_var("count", length as i64)
                        .with_var("max", max_length as i64),
                );
            }
        }

        if let Some(min_length) = self.min_length {
            if let Some(length) = value_length(value)
                && length < min_length
            {
                errors.push(
                    Error::invalid(name, "is too short")
                        .with_var("count", length as i64)
                        .with_var("min", min_length as i64),
                );
            }
        }

        if let (Some(min), Some(number)) = (self.min, value_as_f64(value)) {
            if number < min {
                errors.push(
                    Error::invalid(name, "is less than the minimum")
                        .with_var("value", number)
                        .with_var("min", min),
                );
            }
        }

        if let (Some(max), Some(number)) = (self.max, value_as_f64(value)) {
            if number > max {
                errors.push(
                    Error::invalid(name, "is greater than the maximum")
                        .with_var("value", number)
                        .with_var("max", max),
                );
            }
        }

        if let Some(values) = &self.values {
            let coerced = FieldValue::String(value.to_string());
            if !values.iter().any(|v| *v == coerced || v == value) {
                errors.push(
                    Error::invalid(name, "is not an allowed value").with_var("value", coerced),
                );
            }
        }

        errors.into_result()
    }
}

/// A nil value: absent, null, or an empty string, which Ash treats as nil for
/// most purposes.
pub fn is_nil(value: &FieldValue) -> bool {
    match value {
        FieldValue::Null | FieldValue::NotSet => true,
        FieldValue::String(s) => s.is_empty(),
        _ => false,
    }
}

fn value_length(value: &FieldValue) -> Option<usize> {
    match value {
        FieldValue::String(s) => Some(s.chars().count()),
        FieldValue::Array(v) => Some(v.len()),
        FieldValue::Binary(v) => Some(v.len()),
        FieldValue::Object(v) => Some(v.len()),
        _ => None,
    }
}

fn value_as_f64(value: &FieldValue) -> Option<f64> {
    match value {
        FieldValue::I64(v) => Some(*v as f64),
        FieldValue::I32(v) => Some(*v as f64),
        FieldValue::U64(v) => Some(*v as f64),
        FieldValue::U32(v) => Some(*v as f64),
        FieldValue::F64(v) => Some(*v),
        _ => None,
    }
}

/// One declared attribute of a resource.
#[derive(Clone)]
pub struct Attribute {
    name: String,
    column: String,
    ty: AttributeType,
    constraints: Constraints,
    default: Option<FieldValue>,
    default_fn: Option<Arc<dyn Fn() -> FieldValue + Send + Sync>>,
    description: Option<String>,
    primary_key: bool,
    sensitive: bool,
    public: bool,
    filterable: bool,
    sortable: bool,
    writable: bool,
    default_writable: bool,
}

impl std::fmt::Debug for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Attribute")
            .field("name", &self.name)
            .field("column", &self.column)
            .field("ty", &self.ty)
            .field("constraints", &self.constraints)
            .field("primary_key", &self.primary_key)
            .field("writable", &self.writable)
            .field("default_fn", &self.default_fn.as_ref().map(|_| "<fn>"))
            .finish_non_exhaustive()
    }
}

impl Attribute {
    pub fn new(name: &str, ty: AttributeType) -> Self {
        Self {
            name: name.to_string(),
            column: name.to_string(),
            ty,
            constraints: Constraints::new(),
            default: None,
            default_fn: None,
            description: None,
            primary_key: false,
            sensitive: false,
            public: true,
            filterable: true,
            sortable: true,
            writable: true,
            default_writable: true,
        }
    }

    pub fn string(name: &str) -> Self {
        Self::new(name, AttributeType::String)
    }

    pub fn text(name: &str) -> Self {
        Self::new(name, AttributeType::Text)
    }

    pub fn integer(name: &str) -> Self {
        Self::new(name, AttributeType::Integer)
    }

    pub fn float(name: &str) -> Self {
        Self::new(name, AttributeType::Float)
    }

    pub fn boolean(name: &str) -> Self {
        Self::new(name, AttributeType::Boolean)
    }

    pub fn uuid(name: &str) -> Self {
        Self::new(name, AttributeType::Uuid)
    }

    pub fn ulid(name: &str) -> Self {
        Self::new(name, AttributeType::Ulid)
    }

    pub fn timestamp(name: &str) -> Self {
        Self::new(name, AttributeType::Timestamp)
    }

    pub fn date(name: &str) -> Self {
        Self::new(name, AttributeType::Date)
    }

    pub fn json(name: &str) -> Self {
        Self::new(name, AttributeType::Json)
    }

    pub fn json_value(name: &str) -> Self {
        Self::new(name, AttributeType::JsonValue)
    }

    pub fn binary(name: &str) -> Self {
        Self::new(name, AttributeType::Binary)
    }

    pub fn enumeration(name: &str, options: &[&str]) -> Self {
        Self::new(
            name,
            AttributeType::Enum(options.iter().map(|s| s.to_string()).collect()),
        )
        .constraints(
            Constraints::new().values(
                options
                    .iter()
                    .map(|s| FieldValue::String(s.to_string()))
                    .collect(),
            ),
        )
    }

    /// Mark this attribute as the resource's primary key.
    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self.constraints.allow_nil = false;
        self
    }

    pub fn with_constraints(mut self, callback: impl FnOnce(Constraints) -> Constraints) -> Self {
        self.constraints = callback(self.constraints);
        self
    }

    /// Store the value in a differently named column.
    pub fn column(mut self, column: &str) -> Self {
        self.column = column.to_string();
        self
    }

    pub fn constraints(mut self, constraints: Constraints) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn optional(mut self) -> Self {
        self.constraints.allow_nil = true;
        self
    }

    pub fn required(mut self) -> Self {
        self.constraints.allow_nil = false;
        self
    }

    pub fn default(mut self, value: impl Into<FieldValue>) -> Self {
        self.default = Some(self.ty.coerce(value.into()));
        self
    }

    pub fn default_fn<F: Fn() -> FieldValue + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.default_fn = Some(Arc::new(f));
        self
    }

    pub fn describe(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Hide the value from default reads, the way Ash's `sensitive? true` does.
    /// Sensitive attributes can still be read when explicitly requested.
    pub fn sensitive(mut self) -> Self {
        self.sensitive = true;
        self
    }

    /// Hide the value from action output entirely.
    pub fn private(mut self) -> Self {
        self.public = false;
        self.sensitive = true;
        self
    }

    pub fn not_public(mut self) -> Self {
        self.public = false;
        self
    }

    pub fn filterable(mut self) -> Self {
        self.filterable = true;
        self
    }

    pub fn not_filterable(mut self) -> Self {
        self.filterable = false;
        self
    }

    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    pub fn not_sortable(mut self) -> Self {
        self.sortable = false;
        self
    }

    /// Exclude from the accept list of every create and update action.
    pub fn read_only(mut self) -> Self {
        self.writable = false;
        self.default_writable = false;
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn column_name(&self) -> &str {
        &self.column
    }

    pub fn ty(&self) -> &AttributeType {
        &self.ty
    }

    pub fn constraint(&self) -> &Constraints {
        &self.constraints
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn is_primary_key(&self) -> bool {
        self.primary_key
    }

    pub fn is_sensitive(&self) -> bool {
        self.sensitive
    }

    pub fn is_public(&self) -> bool {
        self.public
    }

    pub fn is_filterable(&self) -> bool {
        self.filterable
    }

    pub fn is_sortable(&self) -> bool {
        self.sortable
    }

    pub fn is_writable(&self) -> bool {
        self.writable
    }

    pub fn allows_nil(&self) -> bool {
        self.constraints.allow_nil
    }

    /// The default exactly as declared, without falling back to the type's
    /// natural starting value. This is what a schema builder needs, since only
    /// an explicit default belongs on the column.
    pub fn declared_default(&self) -> Option<FieldValue> {
        self.default.clone()
    }

    /// The value a new record starts with: the explicit default if one was
    /// declared, otherwise the type's natural starting value.
    pub fn starting_value(&self) -> Option<FieldValue> {
        if let Some(default) = &self.default {
            return Some(default.clone());
        }
        if let Some(f) = &self.default_fn {
            return Some(self.ty.coerce(f()));
        }
        self.ty.implicit_default()
    }
}
