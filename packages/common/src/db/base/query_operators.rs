use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Operator {
    Equal,
    NotEqual,
    Greater,
    NotGreater,
    GreaterOrEqual,
    NotGreaterOrEqual,
    Less,
    LessOrEqual,
    NotLess,
    NotLessOrEqual,
    Like,
    NotLike,
    Null,
    NotNull,
    In,
    NotIn,
}

impl Operator {
    pub fn as_clause(&self, column: &str, placeholder: &str) -> String {
        match &self {
            Self::Equal => format!("{column} = {placeholder}"),
            Self::NotEqual => format!("{column} <> {placeholder}"),
            Self::Greater => format!("{column} > {placeholder}"),
            Self::NotGreater => format!("NOT {column} >= {placeholder}"),
            Self::GreaterOrEqual => format!("{column} >= {placeholder}"),
            Self::NotGreaterOrEqual => format!("{column} >= {placeholder}"),
            Self::Less => format!("{column} < {placeholder}"),
            Self::NotLess => format!("NOT {column} < {placeholder}"),
            Self::LessOrEqual => format!("{column} <= {placeholder}"),
            Self::NotLessOrEqual => format!("NOT {column} <= {placeholder}"),
            Self::Like => format!("{column} LIKE {placeholder}"),
            Self::NotLike => format!("NOT {column} LIKE {placeholder}"),
            Self::Null => format!("{column} IS NULL",),
            Self::NotNull => format!("{column} IS NOT NULL",),
            Self::In => format!("{column} IN ({placeholder})"),
            Self::NotIn => format!("{column} NOT IN ({placeholder})"),
        }
    }
}
