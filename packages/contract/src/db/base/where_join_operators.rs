use super::query_conditions::Condition;

#[derive(Debug, Clone)]
pub enum WhereJoinOperator {
    None(Condition),
    And(Condition),
    Or(Condition),
}

impl WhereJoinOperator {
    pub fn as_clause(&self, existing_wheres: &str, condition: &str) -> String {
        let join = match &self {
            Self::And(_) => "AND",
            Self::Or(_) => "OR",
            Self::None(_) => "",
        };

        format!("{} {} {}", existing_wheres, join, condition)
    }

    pub fn condition(&self) -> &Condition {
        match self {
            Self::And(c) | Self::Or(c) | Self::None(c) => c,
        }
    }
}
