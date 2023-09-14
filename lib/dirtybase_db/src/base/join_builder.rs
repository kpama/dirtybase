use super::query_join_types::JoinType;

#[derive(Debug, Clone, PartialEq)]
pub struct JoinQueryBuilder {
    table: String,
    join_clause: String,
    select_columns: Option<Vec<String>>,
    join_type: JoinType,
}

impl JoinQueryBuilder {
    pub fn new<T: ToString>(
        table: &str,
        left_table: &str,
        operator: &str,
        right_table: &str,
        join_type: JoinType,
        select_columns: Option<&[T]>,
    ) -> Self {
        Self {
            table: table.to_owned(),
            join_clause: format!("{} {} {}", left_table, operator, right_table),
            join_type,
            select_columns: select_columns
                .map(|columns| columns.iter().map(|f| f.to_string()).collect()),
        }
    }

    pub fn select(&mut self, column: &str) -> &mut Self {
        if self.select_columns.is_none() {
            self.select_columns = Some(Vec::new());
        }

        self.select_columns
            .as_mut()
            .unwrap()
            .push(column.to_owned());

        self
    }

    pub fn select_multiple(&mut self, columns: Vec<String>) -> &mut Self {
        if self.select_columns.is_none() {
            self.select_columns = Some(Vec::new());
        }

        self.select_columns.as_mut().unwrap().extend(columns);

        self
    }

    pub fn select_columns(&self) -> &Option<Vec<String>> {
        &self.select_columns
    }

    pub fn join_clause(&self) -> &str {
        &self.join_clause
    }

    pub fn table(&self) -> &str {
        &self.table
    }

    pub fn join_type(&self) -> &JoinType {
        &self.join_type
    }
}
