use dirtybase_db::field_values::FieldValue;

use crate::attribute::AttributeType;

/// Which summary function an aggregate applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateKind {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

impl AggregateKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Count => "count",
            Self::Sum => "sum",
            Self::Avg => "avg",
            Self::Min => "min",
            Self::Max => "max",
        }
    }
}

/// A summary value computed across a whole set of records, either on the
/// resource being read or across one of its relationships.
///
/// This is Ash's `aggregate`. Declaring one is how a resource publishes a
/// `count` or `sum` that a caller can ask for by name.
#[derive(Debug, Clone)]
pub struct Aggregate {
    name: String,
    kind: AggregateKind,
    attribute: String,
    /// Empty means aggregate over the records themselves rather than a
    /// relationship.
    relationship: Option<String>,
    ty: AttributeType,
    public: bool,
    description: Option<String>,
}

impl Aggregate {
    pub fn new(name: &str, kind: AggregateKind, attribute: &str) -> Self {
        Self {
            name: name.to_string(),
            kind,
            attribute: attribute.to_string(),
            relationship: None,
            ty: match kind {
                AggregateKind::Count | AggregateKind::Sum => AttributeType::Integer,
                _ => AttributeType::Float,
            },
            public: true,
            description: None,
        }
    }

    pub fn count(name: &str, attribute: &str) -> Self {
        Self::new(name, AggregateKind::Count, attribute)
    }

    pub fn sum(name: &str, attribute: &str) -> Self {
        Self::new(name, AggregateKind::Sum, attribute)
    }

    pub fn avg(name: &str, attribute: &str) -> Self {
        Self::new(name, AggregateKind::Avg, attribute)
    }

    pub fn min(name: &str, attribute: &str) -> Self {
        Self::new(name, AggregateKind::Min, attribute)
    }

    pub fn max(name: &str, attribute: &str) -> Self {
        Self::new(name, AggregateKind::Max, attribute)
    }

    /// Aggregate over a relationship instead of the records themselves.
    pub fn over_relationship(mut self, relationship: &str) -> Self {
        self.relationship = Some(relationship.to_string());
        self
    }

    pub fn attribute_type(mut self, ty: AttributeType) -> Self {
        self.ty = ty;
        self
    }

    pub fn private(mut self) -> Self {
        self.public = false;
        self
    }

    pub fn describe(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> AggregateKind {
        self.kind
    }

    pub fn attribute(&self) -> &str {
        &self.attribute
    }

    pub fn relationship(&self) -> Option<&str> {
        self.relationship.as_deref()
    }

    pub fn ty(&self) -> &AttributeType {
        &self.ty
    }

    pub fn is_public(&self) -> bool {
        self.public
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Apply the aggregate's function to a set of values.
    ///
    /// The data layer uses this for aggregates that can be answered in Rust,
    /// which covers everything except the SQL-side ones it chooses to push down.
    pub fn apply(&self, values: &[FieldValue]) -> Option<FieldValue> {
        let present: Vec<&FieldValue> = values
            .iter()
            .filter(|value| !crate::attribute::is_nil(value))
            .collect();

        if present.is_empty() {
            return match self.kind {
                AggregateKind::Count => Some(FieldValue::I64(0)),
                _ => None,
            };
        }

        let number = |value: &FieldValue| -> Option<f64> {
            match value {
                FieldValue::I64(v) => Some(*v as f64),
                FieldValue::I32(v) => Some(*v as f64),
                FieldValue::U64(v) => Some(*v as f64),
                FieldValue::U32(v) => Some(*v as f64),
                FieldValue::F64(v) => Some(*v),
                FieldValue::String(s) => s.parse().ok(),
                _ => None,
            }
        };

        let result = match self.kind {
            AggregateKind::Count => present.len() as f64,
            AggregateKind::Sum => present.iter().filter_map(|v| number(v)).sum(),
            AggregateKind::Avg => {
                let collected: Vec<f64> = present.iter().filter_map(|v| number(v)).collect();
                if collected.is_empty() {
                    return None;
                }
                collected.iter().sum::<f64>() / collected.len() as f64
            }
            AggregateKind::Min => present
                .iter()
                .filter_map(|v| number(v))
                .fold(f64::INFINITY, |acc, v| if v < acc { v } else { acc }),
            AggregateKind::Max => present
                .iter()
                .filter_map(|v| number(v))
                .fold(f64::NEG_INFINITY, |acc, v| if v > acc { v } else { acc }),
        };

        Some(match self.ty {
            AttributeType::Integer => FieldValue::I64(result as i64),
            _ => FieldValue::F64(result),
        })
    }
}
