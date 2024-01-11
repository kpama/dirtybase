#[derive(Debug, PartialEq, Clone)]
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
            Self::Equal => format!("{} = {}", column, placeholder),
            Self::NotEqual => format!("{} <> {}", column, placeholder),
            Self::Greater => format!("{} > {}", column, placeholder),
            Self::NotGreater => format!("NOT {} >= {}", column, placeholder),
            Self::GreaterOrEqual => format!("{} >= {}", column, placeholder),
            Self::NotGreaterOrEqual => format!("{} >= {}", column, placeholder),
            Self::Less => format!("{} < {}", column, placeholder),
            Self::NotLess => format!("NOT {} < {}", column, placeholder),
            Self::LessOrEqual => format!("{} <= {}", column, placeholder),
            Self::NotLessOrEqual => format!("NOT {} <= {}", column, placeholder),
            Self::Like => format!("{} LIKE {}", column, placeholder),
            Self::NotLike => format!("NOT {} LIKE {}", column, placeholder),
            Self::Null => format!("{} IS NULL", column),
            Self::NotNull => format!("{} IS NOT NULL", column),
            Self::In => format!("{} IN ({})", column, placeholder),
            Self::NotIn => format!("{} NOT IN ({})", column, placeholder),
        }
    }
}
