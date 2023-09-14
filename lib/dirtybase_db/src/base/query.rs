use crate::entity::user::UserEntity;

use super::{
    join_builder::JoinQueryBuilder, query_conditions::Condition, query_join_types::JoinType,
    query_operators::Operator, table::DELETED_AT_FIELD, where_join_operators::WhereJoinOperator,
};
use dirtybase_db_types::{field_values::FieldValue, types::ColumnAndValue, TableEntityTrait};
use std::{collections::HashMap, fmt::Display};

#[derive(Debug)]
pub enum WhereJoin {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryAction {
    Query {
        columns: Option<Vec<String>>,
        select_all: bool,
    },
    Create {
        rows: Vec<ColumnAndValue>,
        do_soft_insert: bool,
    },
    Update(HashMap<String, FieldValue>),
    Delete,
}

impl Display for QueryAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                QueryAction::Create {
                    rows: _,
                    do_soft_insert: _,
                } => "Create",
                QueryAction::Query {
                    columns: _,
                    select_all: _,
                } => "Query",
                QueryAction::Update(_) => "Update",
                QueryAction::Delete => "Delete",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct QueryBuilder {
    where_clauses: Vec<WhereJoinOperator>,
    tables: Vec<String>,
    joins: Option<Vec<JoinQueryBuilder>>,
    action: QueryAction,
}

impl QueryBuilder {
    pub fn new(tables: Vec<String>, action: QueryAction) -> Self {
        Self {
            where_clauses: Vec::new(),
            tables,
            joins: None,
            action,
        }
    }

    pub fn action(&self) -> &QueryAction {
        &self.action
    }

    pub fn tables(&self) -> &Vec<String> {
        &self.tables
    }

    pub fn all_columns(&self) -> bool {
        match self.action {
            QueryAction::Query {
                select_all,
                columns: _,
            } => select_all,
            _ => false,
        }
    }

    pub fn select_all(&mut self) -> &mut Self {
        match &mut self.action {
            QueryAction::Query {
                select_all,
                columns: _,
            } => *select_all = true,
            _ => (),
        }
        self
    }

    pub fn joins(&self) -> &Option<Vec<JoinQueryBuilder>> {
        &self.joins
    }

    /// Set a column/value for update
    pub fn set_column<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        match &mut self.action {
            QueryAction::Update(columns) => {
                columns.insert(column.to_string(), value.into());
            }
            _ => (),
        }
        self
    }

    /// Set multiple column/value for update
    pub fn set_columns(&mut self, column_and_values: ColumnAndValue) -> &mut Self {
        match &mut self.action {
            QueryAction::Update(columns) => {
                columns.extend(column_and_values);
            }
            _ => (),
        }
        self
    }

    /// Set multiple rows for insert/create
    pub fn set_insert_rows(&mut self, rows_to_insert: Vec<ColumnAndValue>) -> &mut Self {
        match &mut self.action {
            QueryAction::Create {
                rows,
                do_soft_insert: _,
            } => {
                *rows = rows_to_insert;
            }
            _ => (),
        }
        self
    }

    /// Returns a reference to the `where` clauses vec
    pub fn where_clauses(&self) -> &Vec<WhereJoinOperator> {
        &self.where_clauses
    }

    /// Returns a mut reference to the `where` clauses vec
    pub fn where_clauses_mut(&mut self) -> &mut Vec<WhereJoinOperator> {
        &mut self.where_clauses
    }

    /// Replaces the existing `where` clauses vec with the provided one
    pub fn set_where_clauses(&mut self, where_classes: Vec<WhereJoinOperator>) -> &mut Self {
        self.where_clauses = where_classes;
        self
    }

    /// Adds a column that should be selected
    pub fn select<T: ToString>(&mut self, column: T) -> &mut Self {
        match &mut self.action {
            QueryAction::Query {
                columns,
                select_all,
            } => {
                *select_all = false;
                if let Some(list) = columns {
                    list.push(column.to_string())
                } else {
                    *columns = Some(vec![column.to_string()]);
                }
            }
            _ => (),
        }

        self
    }

    /// Adds a table to the list of tables to select from
    pub fn select_table<T: TableEntityTrait>(&mut self) -> &mut Self {
        self.select_multiple(&T::table_column_full_names())
    }

    /// Adds multiple columns to be selected
    pub fn select_multiple<T: ToString>(&mut self, columns_to_select: &[T]) -> &mut Self {
        match &mut self.action {
            QueryAction::Query {
                columns,
                select_all,
            } => {
                *select_all = false;
                if let Some(list) = columns {
                    list.extend(columns_to_select.iter().map(|c| c.to_string()))
                } else {
                    *columns = Some(
                        columns_to_select
                            .iter()
                            .map(|c| c.to_string())
                            .collect::<Vec<String>>(),
                    );
                }
            }
            _ => (),
        }

        self
    }

    /// `WHERE` column equals value
    pub fn eq<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::Equal, value, None)
    }

    /// `AND WHERE` column equals value
    pub fn and_eq<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::Equal, value, Some(WhereJoin::And))
    }

    /// `OR WHERE` column equals value
    pub fn or_eq<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::Equal, value, Some(WhereJoin::Or))
    }

    /// `NOT EQUAL` column not equal value
    pub fn not_eq<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotEqual, value, None)
    }

    /// `AND NOT EQUAL` column not equal value
    pub fn and_not_eq<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::NotEqual, value, Some(WhereJoin::And))
    }

    /// `OR NOT EQUAL` column not equal value
    pub fn or_not_eq<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::NotEqual, value, Some(WhereJoin::Or))
    }

    /// `GREATER THAN` column is greater than value
    pub fn gt<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::Greater, value, None)
    }

    /// `AND GREATER THAN` column is greater than value
    pub fn and_gt<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::Greater, value, Some(WhereJoin::And))
    }

    /// `OR GREATER THAN` column is greater than value
    pub fn or_gt<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::Greater, value, Some(WhereJoin::Or))
    }

    /// `NOT GREATER THAN` column is not greater than value
    pub fn ngt<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotGreater, value, None)
    }

    /// `AND NOT GREATER THAN` column is not greater than value
    pub fn and_ngt<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotGreater, value, Some(WhereJoin::And))
    }

    /// `OR NOT GREATER THAN` column is not greater than value
    pub fn or_ngt<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotGreater, value, Some(WhereJoin::Or))
    }

    /// `GREATER THAN OR EQUAL TO` column is greater than or equal the value
    pub fn gt_or_eq<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::GreaterOrEqual, value, None)
    }

    /// `AND GREATER THAN OR EQUAL TO` column is greater than or equal the value
    pub fn and_gt_or_eq<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        self.where_operator(
            column,
            Operator::GreaterOrEqual,
            value,
            Some(WhereJoin::And),
        )
    }

    /// `OR GREATER THAN OR EQUAL TO` column is greater than or equal the value
    pub fn or_gt_or_eq<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::GreaterOrEqual, value, Some(WhereJoin::Or))
    }

    /// `NOT GREATER THAN OR EQUAL TO` column is not greater than or equal the value
    pub fn not_gt_or_eq<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::NotGreaterOrEqual, value, None)
    }

    /// `AND NOT GREATER THAN OR EQUAL TO` column is not greater than or equal the value
    pub fn and_not_gt_or_eq<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        self.where_operator(
            column,
            Operator::NotGreaterOrEqual,
            value,
            Some(WhereJoin::And),
        )
    }

    /// `OR NOT GREATER THAN OR EQUAL TO` column is not greater than or equal the value
    pub fn or_not_gt_or_eq<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        self.where_operator(
            column,
            Operator::NotGreaterOrEqual,
            value,
            Some(WhereJoin::Or),
        )
    }

    /// `LESS THAN` column is less than the value
    pub fn le<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::Less, value, None)
    }

    /// `AND LESS THAN` column is less than the value
    pub fn and_le<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::Less, value, Some(WhereJoin::And))
    }

    /// `OR LESS THAN` column is less than the value
    pub fn or_le<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::Less, value, Some(WhereJoin::Or))
    }

    /// `LESS THAN OR EQUAL` column is less than or equal value
    pub fn le_or_eq<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::LessOrEqual, value, None)
    }

    /// `AND LESS THAN OR EQUAL` column is less than or equal value
    pub fn and_le_or_eq<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::LessOrEqual, value, Some(WhereJoin::And))
    }

    /// `OR LESS THAN OR EQUAL` column is less than or equal value
    pub fn or_le_or_eq<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::LessOrEqual, value, Some(WhereJoin::Or))
    }

    pub fn not_le<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotLess, value, None)
    }

    pub fn and_nle<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotLess, value, Some(WhereJoin::And))
    }

    pub fn or_nle<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotLess, value, Some(WhereJoin::Or))
    }

    pub fn not_le_or_eq<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::NotLessOrEqual, value, None)
    }

    pub fn and_not_le_or_eq<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        self.where_operator(
            column,
            Operator::NotLessOrEqual,
            value,
            Some(WhereJoin::And),
        )
    }

    pub fn or_not_le_or_eq<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::NotLessOrEqual, value, Some(WhereJoin::Or))
    }

    pub fn like<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::Like, value, None)
    }

    pub fn and_like<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::Like, value, Some(WhereJoin::And))
    }

    pub fn or_like<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::Like, value, Some(WhereJoin::Or))
    }

    pub fn not_like<T: Into<FieldValue>, C: ToString>(&mut self, column: C, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotLike, value, None)
    }

    pub fn and_not_like<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::NotLike, value, Some(WhereJoin::And))
    }

    pub fn or_not_like<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::NotLike, value, Some(WhereJoin::Or))
    }

    pub fn is_null<C: ToString>(&mut self, column: C) -> &mut Self {
        self.where_operator(column, Operator::Null, FieldValue::Null, None)
    }

    pub fn and_is_null<C: ToString>(&mut self, column: C) -> &mut Self {
        self.where_operator(
            column,
            Operator::Null,
            FieldValue::Null,
            Some(WhereJoin::And),
        )
    }

    pub fn or_is_null<C: ToString>(&mut self, column: C) -> &mut Self {
        self.where_operator(
            column,
            Operator::Null,
            FieldValue::Null,
            Some(WhereJoin::Or),
        )
    }

    // TODO: Test this feature. Also allow optional prefix?
    pub fn without_trash(&mut self) -> &mut Self {
        self.is_null(DELETED_AT_FIELD)
    }

    pub fn without_table_trash<T: TableEntityTrait>(&mut self) -> &mut Self {
        if let Some(field) = T::deleted_at_column() {
            self.is_null(T::prefix_with_tbl(field));
        }
        self
    }

    // TODO: Test this feature. Also allow optional prefix?
    pub fn with_trash(&mut self) -> &mut Self {
        self.is_null(DELETED_AT_FIELD)
            .or_is_not_null(DELETED_AT_FIELD)
    }

    pub fn is_not_null<C: ToString>(&mut self, column: C) -> &mut Self {
        self.where_operator(column, Operator::NotNull, FieldValue::Null, None)
    }

    pub fn and_is_not_null<C: ToString>(&mut self, column: C) -> &mut Self {
        self.where_operator(
            column,
            Operator::NotNull,
            FieldValue::Null,
            Some(WhereJoin::And),
        )
    }

    pub fn or_is_not_null<C: ToString>(&mut self, column: C) -> &mut Self {
        self.where_operator(
            column,
            Operator::NotNull,
            FieldValue::Null,
            Some(WhereJoin::Or),
        )
    }

    pub fn is_in<T: Into<FieldValue> + IntoIterator, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::In, value, None)
    }

    pub fn and_is_in<T: Into<FieldValue> + IntoIterator, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::In, value, Some(WhereJoin::And))
    }

    pub fn or_is_in<T: Into<FieldValue> + IntoIterator, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::In, value, Some(WhereJoin::Or))
    }

    pub fn is_not_in<T: Into<FieldValue> + IntoIterator, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::NotIn, value, None)
    }

    pub fn and_is_not_in<T: Into<FieldValue> + IntoIterator, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::NotIn, value, Some(WhereJoin::And))
    }

    pub fn or_is_not_in<T: Into<FieldValue> + IntoIterator, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::NotIn, value, Some(WhereJoin::Or))
    }

    pub fn between<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        first: T,
        last: T,
    ) -> &mut Self {
        self.gt_or_eq(column.to_string(), first)
            .and_le_or_eq(column, last)
    }

    pub fn and_between<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        first: T,
        last: T,
    ) -> &mut Self {
        self.and_gt_or_eq(column.to_string(), first)
            .and_le_or_eq(column, last)
    }

    pub fn or_between<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        first: T,
        last: T,
    ) -> &mut Self {
        self.or_gt_or_eq(column.to_string(), first)
            .and_le_or_eq(column, last)
    }

    pub fn not_between<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        first: T,
        last: T,
    ) -> &mut Self {
        self.not_gt_or_eq(column.to_string(), first)
            .and_not_le_or_eq(column, last)
    }

    pub fn and_not_between<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        first: T,
        last: T,
    ) -> &mut Self {
        self.and_not_gt_or_eq(column.to_string(), first)
            .and_not_le_or_eq(column, last)
    }

    pub fn or_not_between<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        first: T,
        last: T,
    ) -> &mut Self {
        self.or_not_gt_or_eq(column.to_string(), first)
            .and_not_le_or_eq(column, last)
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

    pub fn where_operator<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        operator: Operator,
        value: T,
        and_or: Option<WhereJoin>,
    ) -> &mut Self {
        let condition = Condition::new(column, operator, value.into());
        match and_or {
            Some(j) => match j {
                WhereJoin::And => {
                    if self.where_clauses.is_empty() {
                        self.first_or_and(condition)
                    } else {
                        self.and_where(condition)
                    }
                }
                WhereJoin::Or => {
                    if self.where_clauses.is_empty() {
                        self.first_or_and(condition)
                    } else {
                        self.or_where(condition)
                    }
                }
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

    pub fn with_creator<L: TableEntityTrait>(&mut self, prefix: Option<&str>) -> &mut Self {
        if let Some(field) = L::creator_id_column() {
            self.left_join_table_and_select::<UserEntity, L>(
                UserEntity::id_column().unwrap(),
                &L::prefix_with_tbl(&field),
                prefix,
            );
        }
        self
    }

    pub fn join<T: ToString>(
        &mut self,
        table: &str,
        left_table: &str,
        operator: &str,
        right_table: &str,
        join_type: JoinType,
        select_columns: Option<&[T]>,
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
        self.join::<String>(
            table,
            left_table,
            operator,
            right_table,
            JoinType::Inner,
            None,
        )
    }
    pub fn inner_join_table<L: TableEntityTrait, R: TableEntityTrait>(
        &mut self,
        left_field: &str,
        right_field: &str,
    ) -> &mut Self {
        self.inner_join(
            L::table_name(),
            &L::prefix_with_tbl(left_field),
            "=",
            &R::prefix_with_tbl(right_field),
        )
    }

    pub fn inner_join_and_select<T: ToString>(
        &mut self,
        table: &str,
        left_table: &str,
        operator: &str,
        right_table: &str,
        select_columns: &[T],
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

    pub fn inner_join_table_and_select<L: TableEntityTrait, R: TableEntityTrait>(
        &mut self,
        left_field: &str,
        right_field: &str,
        left_tbl_columns_prefix: Option<&str>,
    ) -> &mut Self {
        self.inner_join_and_select(
            L::table_name(),
            &L::prefix_with_tbl(left_field),
            "=",
            &R::prefix_with_tbl(right_field),
            &L::column_aliases(left_tbl_columns_prefix),
        )
    }

    pub fn left_join(
        &mut self,
        table: &str,
        left_table: &str,
        operator: &str,
        right_table: &str,
    ) -> &mut Self {
        self.join::<String>(
            table,
            left_table,
            operator,
            right_table,
            JoinType::Left,
            None,
        )
    }

    pub fn left_join_table<L: TableEntityTrait, R: TableEntityTrait>(
        &mut self,
        left_field: &str,
        right_field: &str,
    ) -> &mut Self {
        self.left_join(
            L::table_name(),
            &L::prefix_with_tbl(left_field),
            "=",
            &R::prefix_with_tbl(right_field),
        )
    }

    pub fn left_join_and_select<T: ToString>(
        &mut self,
        table: &str,
        left_table: &str,
        operator: &str,
        right_table: &str,
        select_columns: &[T],
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

    pub fn left_join_table_and_select<L: TableEntityTrait, R: TableEntityTrait>(
        &mut self,
        left_field: &str,
        right_field: &str,
        left_tbl_columns_prefix: Option<&str>,
    ) -> &mut Self {
        self.left_join_and_select(
            L::table_name(),
            &L::prefix_with_tbl(left_field),
            "=",
            &R::prefix_with_tbl(right_field),
            &L::column_aliases(left_tbl_columns_prefix),
        )
    }

    pub fn right_join(
        &mut self,
        table: &str,
        left_table: &str,
        operator: &str,
        right_table: &str,
    ) -> &mut Self {
        self.join::<String>(
            table,
            left_table,
            operator,
            right_table,
            JoinType::Right,
            None,
        )
    }

    pub fn right_join_table<L: TableEntityTrait, R: TableEntityTrait>(
        &mut self,
        left_field: &str,
        right_field: &str,
    ) -> &mut Self {
        self.right_join(
            L::table_name(),
            &L::prefix_with_tbl(left_field),
            "=",
            &R::prefix_with_tbl(right_field),
        )
    }

    pub fn right_join_and_select<T: ToString>(
        &mut self,
        table: &str,
        left_table: &str,
        operator: &str,
        right_table: &str,
        select_columns: &[T],
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

    pub fn right_join_table_and_select<L: TableEntityTrait, R: TableEntityTrait>(
        &mut self,
        left_field: &str,
        right_field: &str,
        left_tbl_columns_prefix: Option<&str>,
    ) -> &mut Self {
        self.right_join_and_select(
            L::table_name(),
            &L::prefix_with_tbl(left_field),
            "=",
            &R::prefix_with_tbl(right_field),
            &L::column_aliases(left_tbl_columns_prefix),
        )
    }
}
