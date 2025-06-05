#[derive(Debug, Clone, PartialEq)]
pub enum Aggregate {
    Count,
    Max,
    Min,
    Sum,
    Avg,
}
