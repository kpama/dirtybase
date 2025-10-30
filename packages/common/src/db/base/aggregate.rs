use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Aggregate {
    Count,
    Max,
    Min,
    Sum,
    Avg,
}
