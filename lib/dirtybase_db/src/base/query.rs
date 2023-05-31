use std::collections::HashMap;

use super::{
    insert_values::InsertValue, join_builder::JoinQueryBuilder, query_conditions::Condition,
    query_join_types::JoinType, query_operators::Operator, query_values::Value,
    where_join_operators::WhereJoinOperator,
};

#[derive(Debug)]
pub enum WhereJoin {
    And,
    Or,
}

#[derive(Debug)]
pub struct QueryBuilder {
    where_clauses: Vec<WhereJoinOperator>,
    tables: Vec<String>,
    select_columns: Option<Vec<String>>,
    set_columns: Option<HashMap<String, InsertValue>>,
    joins: Option<Vec<JoinQueryBuilder>>,
}

impl QueryBuilder {
    pub fn new(tables: Vec<String>) -> Self {
        Self {
            where_clauses: Vec::new(),
            tables,
            select_columns: None,
            set_columns: None,
            joins: None,
        }
    }

    pub fn tables(&self) -> &Vec<String> {
        &self.tables
    }

    pub fn select_columns(&self) -> &Option<Vec<String>> {
        &self.select_columns
    }

    pub fn set_columns(&self) -> &Option<HashMap<String, InsertValue>> {
        &self.set_columns
    }

    pub fn joins(&self) -> &Option<Vec<JoinQueryBuilder>> {
        &self.joins
    }

    pub fn set<T: Into<InsertValue>>(&mut self, column: &str, value: T) -> &mut Self {
        if self.set_columns.is_none() {
            self.set_columns = Some(HashMap::new());
        }

        if let Some(columns) = &mut self.set_columns {
            columns.insert(column.to_string(), value.into());
        }

        self
    }

    pub fn set_multiple<T: Into<InsertValue>>(
        &mut self,
        column_and_values: HashMap<String, T>,
    ) -> &mut Self {
        if self.set_columns.is_none() {
            self.set_columns = Some(HashMap::new());
        }

        if let Some(columns) = &mut self.set_columns {
            for entry in column_and_values {
                columns.insert(entry.0, entry.1.into());
            }
        }

        self
    }

    pub fn where_clauses(&self) -> &Vec<WhereJoinOperator> {
        &self.where_clauses
    }

    pub fn where_clauses_mut(&mut self) -> &mut Vec<WhereJoinOperator> {
        &mut self.where_clauses
    }

    pub fn set_where_clauses(&mut self, where_classes: Vec<WhereJoinOperator>) -> &mut Self {
        self.where_clauses = where_classes;
        self
    }

    pub fn select(&mut self, column: &str) -> &mut Self {
        if self.select_columns.is_none() {
            self.select_columns = Some(Vec::new());
        }

        if let Some(columns) = &mut self.select_columns {
            columns.push(column.to_owned());
        }

        self
    }

    pub fn select_multiple(&mut self, columns: &[&str]) -> &mut Self {
        if self.select_columns.is_none() {
            self.select_columns = Some(Vec::new());
        }

        if let Some(existing) = &mut self.select_columns {
            existing.extend(
                columns
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>(),
            );
        }

        self
    }

    pub fn eq<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Equal, value, None)
    }

    pub fn and_eq<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Equal, value, Some(WhereJoin::And))
    }

    pub fn or_eq<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Equal, value, Some(WhereJoin::Or))
    }

    pub fn not_eq<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotEqual, value, None)
    }

    pub fn and_not_eq<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotEqual, value, Some(WhereJoin::And))
    }

    pub fn or_not_eq<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotEqual, value, Some(WhereJoin::Or))
    }

    pub fn gt<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Greater, value, None)
    }

    pub fn and_gt<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Greater, value, Some(WhereJoin::And))
    }

    pub fn or_gt<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Greater, value, Some(WhereJoin::Or))
    }

    pub fn ngt<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotGreater, value, None)
    }
    pub fn and_ngt<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotGreater, value, Some(WhereJoin::And))
    }

    pub fn or_ngt<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotGreater, value, Some(WhereJoin::Or))
    }

    pub fn gt_or_eq<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::GreaterOrEqual, value, None)
    }

    pub fn and_gt_or_eq<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(
            column,
            Operator::GreaterOrEqual,
            value,
            Some(WhereJoin::And),
        )
    }
    pub fn or_gt_or_eq<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::GreaterOrEqual, value, Some(WhereJoin::Or))
    }

    pub fn not_gt_or_eq<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotGreaterOrEqual, value, None)
    }

    pub fn and_not_gt_or_eq<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(
            column,
            Operator::NotGreaterOrEqual,
            value,
            Some(WhereJoin::And),
        )
    }

    pub fn or_not_gt_or_eq<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(
            column,
            Operator::NotGreaterOrEqual,
            value,
            Some(WhereJoin::Or),
        )
    }

    pub fn le<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Less, value, None)
    }

    pub fn and_le<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Less, value, Some(WhereJoin::And))
    }
    pub fn or_le<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Less, value, Some(WhereJoin::Or))
    }

    pub fn le_or_eq<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::LessOrEqual, value, None)
    }

    pub fn and_le_or_eq<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::LessOrEqual, value, Some(WhereJoin::And))
    }

    pub fn or_le_or_eq<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::LessOrEqual, value, Some(WhereJoin::Or))
    }

    pub fn not_le<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotLess, value, None)
    }

    pub fn and_nle<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotLess, value, Some(WhereJoin::And))
    }

    pub fn or_nle<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotLess, value, Some(WhereJoin::Or))
    }

    pub fn nle_or_eq<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotLessOrEqual, value, None)
    }

    pub fn and_nle_or_eq<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(
            column,
            Operator::NotLessOrEqual,
            value,
            Some(WhereJoin::And),
        )
    }

    pub fn or_nle_or_eq<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotLessOrEqual, value, Some(WhereJoin::Or))
    }

    pub fn like<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Like, value, None)
    }

    pub fn and_like<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Like, value, Some(WhereJoin::And))
    }

    pub fn or_like<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Like, value, Some(WhereJoin::Or))
    }

    pub fn nlike<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotLike, value, None)
    }

    pub fn and_nlike<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotLike, value, Some(WhereJoin::And))
    }

    pub fn or_nlike<T: Into<Value>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotLike, value, Some(WhereJoin::Or))
    }

    pub fn is_null(&mut self, column: &str) -> &mut Self {
        self.where_operator(column, Operator::Null, Value::Null, None)
    }

    pub fn and_is_null(&mut self, column: &str) -> &mut Self {
        self.where_operator(column, Operator::Null, Value::Null, Some(WhereJoin::And))
    }

    pub fn or_is_null(&mut self, column: &str) -> &mut Self {
        self.where_operator(column, Operator::Null, Value::Null, Some(WhereJoin::Or))
    }

    pub fn is_not_null(&mut self, column: &str) -> &mut Self {
        self.where_operator(column, Operator::NotNull, Value::Null, None)
    }

    pub fn and_is_not_null(&mut self, column: &str) -> &mut Self {
        self.where_operator(column, Operator::NotNull, Value::Null, Some(WhereJoin::And))
    }

    pub fn or_is_not_null(&mut self, column: &str) -> &mut Self {
        self.where_operator(column, Operator::NotNull, Value::Null, Some(WhereJoin::Or))
    }

    pub fn is_in<T: Into<Value> + IntoIterator>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::In, value, None)
    }

    pub fn and_is_in<T: Into<Value> + IntoIterator>(
        &mut self,
        column: &str,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::In, value, Some(WhereJoin::And))
    }

    pub fn or_is_in<T: Into<Value> + IntoIterator>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::In, value, Some(WhereJoin::Or))
    }

    pub fn is_not_in<T: Into<Value> + IntoIterator>(
        &mut self,
        column: &str,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::NotIn, value, None)
    }

    pub fn and_is_not_in<T: Into<Value> + IntoIterator>(
        &mut self,
        column: &str,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::NotIn, value, Some(WhereJoin::And))
    }

    pub fn or_is_not_in<T: Into<Value> + IntoIterator>(
        &mut self,
        column: &str,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::NotIn, value, Some(WhereJoin::Or))
    }

    pub fn between<T: Into<Value>>(&mut self, column: &str, first: T, last: T) -> &mut Self {
        self.gt_or_eq(column, first).and_le_or_eq(column, last)
    }

    pub fn and_between<T: Into<Value>>(&mut self, column: &str, first: T, last: T) -> &mut Self {
        self.and_gt_or_eq(column, first).and_le_or_eq(column, last)
    }

    pub fn or_between<T: Into<Value>>(&mut self, column: &str, first: T, last: T) -> &mut Self {
        self.or_gt_or_eq(column, first).and_le_or_eq(column, last)
    }

    pub fn not_between<T: Into<Value>>(&mut self, column: &str, first: T, last: T) -> &mut Self {
        self.not_gt_or_eq(column, first).and_nle_or_eq(column, last)
    }

    pub fn and_not_between<T: Into<Value>>(
        &mut self,
        column: &str,
        first: T,
        last: T,
    ) -> &mut Self {
        self.and_not_gt_or_eq(column, first)
            .and_nle_or_eq(column, last)
    }

    pub fn or_not_between<T: Into<Value>>(&mut self, column: &str, first: T, last: T) -> &mut Self {
        self.or_not_gt_or_eq(column, first)
            .and_nle_or_eq(column, last)
    }

    pub fn where_(&mut self, where_clause: WhereJoinOperator) -> &mut Self {
        self.where_clauses.push(where_clause);
        self
    }

    fn first_or_and(&mut self, condition: Condition) -> &mut Self {
        if self.where_clauses.is_empty() {
            self.where_(WhereJoinOperator::None(condition))
        } else {
            self.and_where(condition)
        }
    }

    pub fn where_operator<T: Into<Value>>(
        &mut self,
        column: &str,
        operator: Operator,
        value: T,
        and_or: Option<WhereJoin>,
    ) -> &mut Self {
        let condition = Condition::new(column, operator, value);
        match and_or {
            Some(j) => match j {
                WhereJoin::And => self.and_where(condition),
                WhereJoin::Or => self.or_where(condition),
            },
            _ => self.first_or_and(condition),
        }
    }

    fn or_where(&mut self, condition: Condition) -> &mut Self {
        self.where_(WhereJoinOperator::Or(condition))
    }

    fn and_where(&mut self, condition: Condition) -> &mut Self {
        self.where_(WhereJoinOperator::And(condition))
    }

    pub fn join(
        &mut self,
        table: &str,
        left_table: &str,
        operator: &str,
        right_table: &str,
        join_type: JoinType,
        select_columns: Option<&[&str]>,
    ) -> &mut Self {
        if self.joins.is_none() {
            self.joins = Some(Vec::new());
        }

        let join = JoinQueryBuilder::new(
            table,
            left_table,
            operator,
            right_table,
            join_type,
            select_columns,
        );
        self.joins.as_mut().unwrap().push(join);

        self
    }

    pub fn inner_join(
        &mut self,
        table: &str,
        left_table: &str,
        operator: &str,
        right_table: &str,
    ) -> &mut Self {
        self.join(
            table,
            left_table,
            operator,
            right_table,
            JoinType::Inner,
            None,
        )
    }

    pub fn inner_join_and_select(
        &mut self,
        table: &str,
        left_table: &str,
        operator: &str,
        right_table: &str,
        select_columns: &[&str],
    ) -> &mut Self {
        self.join(
            table,
            left_table,
            operator,
            right_table,
            JoinType::Inner,
            Some(select_columns),
        )
    }

    pub fn left_join(
        &mut self,
        table: &str,
        left_table: &str,
        operator: &str,
        right_table: &str,
    ) -> &mut Self {
        self.join(
            table,
            left_table,
            operator,
            right_table,
            JoinType::Left,
            None,
        )
    }

    pub fn left_join_and_select(
        &mut self,
        table: &str,
        left_table: &str,
        operator: &str,
        right_table: &str,
        select_columns: &[&str],
    ) -> &mut Self {
        self.join(
            table,
            left_table,
            operator,
            right_table,
            JoinType::Left,
            Some(select_columns),
        )
    }

    pub fn right_join(
        &mut self,
        table: &str,
        left_table: &str,
        operator: &str,
        right_table: &str,
    ) -> &mut Self {
        self.join(
            table,
            left_table,
            operator,
            right_table,
            JoinType::Right,
            None,
        )
    }

    pub fn right_join_and_select(
        &mut self,
        table: &str,
        left_table: &str,
        operator: &str,
        right_table: &str,
        select_columns: &[&str],
    ) -> &mut Self {
        self.join(
            table,
            left_table,
            operator,
            right_table,
            JoinType::Right,
            Some(select_columns),
        )
    }
}
