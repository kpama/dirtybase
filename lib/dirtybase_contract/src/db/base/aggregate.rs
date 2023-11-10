use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Aggregate {
    Count(String),
    Max(String),
    Min(String),
    Sum(String),
    Avg(String),
}

impl Display for Aggregate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Count(inner) => write!(f, "COUNT({})", inner),
            Self::Max(inner) => write!(f, "MAX({})", inner),
            Self::Min(inner) => write!(f, "MIN({})", inner),
            Self::Sum(inner) => write!(f, "SUM({})", inner),
            Self::Avg(inner) => write!(f, "AVG({})", inner),
        }
    }
}
