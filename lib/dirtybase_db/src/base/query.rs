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

#[derive(Debug, Clone, Copy)]
pub enum QueryAction {
    Query,
    Create,
    Update,
    Delete,
}

impl Display for QueryAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                QueryAction::Create => "Create",
                QueryAction::Query => "Query",
                QueryAction::Update => "Update",
                QueryAction::Delete => "Delete",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct QueryBuilder {
    where_clauses: Vec<WhereJoinOperator>,
    tables: Vec<String>,
    select_columns: Option<Vec<String>>,
    set_columns: Option<HashMap<String, FieldValue>>, // TODO: refactored name !!!
    all_columns: bool,
    joins: Option<Vec<JoinQueryBuilder>>,
    action: QueryAction,
}

impl QueryBuilder {
    pub fn new(tables: Vec<String>, action: QueryAction) -> Self {
        Self {
            where_clauses: Vec::new(),
            tables,
            select_columns: None,
            set_columns: None,
            all_columns: false,
            joins: None,
            action,
        }
    }

    pub fn action(&self) -> QueryAction {
        self.action
    }

    pub fn tables(&self) -> &Vec<String> {
        &self.tables
    }

    pub fn select_columns(&self) -> &Option<Vec<String>> {
        &self.select_columns
    }

    pub fn set_columns(&self) -> &Option<HashMap<String, FieldValue>> {
        &self.set_columns
    }

    pub fn all_columns(&self) -> bool {
        self.all_columns
    }

    pub fn select_all(&mut self) -> &mut Self {
        self.all_columns = true;
        self
    }

    pub fn joins(&self) -> &Option<Vec<JoinQueryBuilder>> {
        &self.joins
    }

    pub fn set<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        if self.set_columns.is_none() {
            self.set_columns = Some(HashMap::new());
        }

        if let Some(columns) = &mut self.set_columns {
            columns.insert(column.to_string(), value.into());
        }

        self
    }

    pub fn set_multiple(&mut self, column_and_values: ColumnAndValue) -> &mut Self {
        if self.set_columns.is_none() {
            self.set_columns = Some(HashMap::new());
        }

        if let Some(columns) = &mut self.set_columns {
            for entry in column_and_values {
                columns.insert(entry.0, entry.1);
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

    pub fn select<T: ToString>(&mut self, column: T) -> &mut Self {
        self.init_select_columns_vec();

        if let Some(columns) = &mut self.select_columns {
            columns.push(column.to_string());
        }

        self
    }

    pub fn select_table<T: TableEntityTrait>(&mut self) -> &mut Self {
        self.select_multiple(&T::table_column_full_names())
    }

    pub fn select_multiple<T: ToString>(&mut self, columns: &[T]) -> &mut Self {
        self.init_select_columns_vec();
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

    // WHERE field equals value
    pub fn eq<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Equal, value, None)
    }

    // AND WHERE field equals value
    pub fn and_eq<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Equal, value, Some(WhereJoin::And))
    }

    // OR WHERE field equals value
    pub fn or_eq<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Equal, value, Some(WhereJoin::Or))
    }

    pub fn not_eq<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotEqual, value, None)
    }

    pub fn and_not_eq<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotEqual, value, Some(WhereJoin::And))
    }

    pub fn or_not_eq<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotEqual, value, Some(WhereJoin::Or))
    }

    pub fn gt<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Greater, value, None)
    }

    pub fn and_gt<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Greater, value, Some(WhereJoin::And))
    }

    pub fn or_gt<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Greater, value, Some(WhereJoin::Or))
    }

    pub fn ngt<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotGreater, value, None)
    }
    pub fn and_ngt<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotGreater, value, Some(WhereJoin::And))
    }

    pub fn or_ngt<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotGreater, value, Some(WhereJoin::Or))
    }

    pub fn gt_or_eq<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::GreaterOrEqual, value, None)
    }

    pub fn and_gt_or_eq<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(
            column,
            Operator::GreaterOrEqual,
            value,
            Some(WhereJoin::And),
        )
    }
    pub fn or_gt_or_eq<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::GreaterOrEqual, value, Some(WhereJoin::Or))
    }

    pub fn not_gt_or_eq<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotGreaterOrEqual, value, None)
    }

    pub fn and_not_gt_or_eq<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(
            column,
            Operator::NotGreaterOrEqual,
            value,
            Some(WhereJoin::And),
        )
    }

    pub fn or_not_gt_or_eq<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(
            column,
            Operator::NotGreaterOrEqual,
            value,
            Some(WhereJoin::Or),
        )
    }

    pub fn le<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Less, value, None)
    }

    pub fn and_le<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Less, value, Some(WhereJoin::And))
    }
    pub fn or_le<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Less, value, Some(WhereJoin::Or))
    }

    pub fn le_or_eq<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::LessOrEqual, value, None)
    }

    pub fn and_le_or_eq<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::LessOrEqual, value, Some(WhereJoin::And))
    }

    pub fn or_le_or_eq<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::LessOrEqual, value, Some(WhereJoin::Or))
    }

    pub fn not_le<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotLess, value, None)
    }

    pub fn and_nle<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotLess, value, Some(WhereJoin::And))
    }

    pub fn or_nle<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotLess, value, Some(WhereJoin::Or))
    }

    pub fn not_le_or_eq<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotLessOrEqual, value, None)
    }

    pub fn and_not_le_or_eq<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(
            column,
            Operator::NotLessOrEqual,
            value,
            Some(WhereJoin::And),
        )
    }

    pub fn or_not_le_or_eq<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotLessOrEqual, value, Some(WhereJoin::Or))
    }

    pub fn like<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Like, value, None)
    }

    pub fn and_like<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Like, value, Some(WhereJoin::And))
    }

    pub fn or_like<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::Like, value, Some(WhereJoin::Or))
    }

    pub fn not_like<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotLike, value, None)
    }

    pub fn and_not_like<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotLike, value, Some(WhereJoin::And))
    }

    pub fn or_not_like<T: Into<FieldValue>>(&mut self, column: &str, value: T) -> &mut Self {
        self.where_operator(column, Operator::NotLike, value, Some(WhereJoin::Or))
    }

    pub fn is_null(&mut self, column: &str) -> &mut Self {
        self.where_operator(column, Operator::Null, FieldValue::Null, None)
    }

    pub fn and_is_null(&mut self, column: &str) -> &mut Self {
        self.where_operator(
            column,
            Operator::Null,
            FieldValue::Null,
            Some(WhereJoin::And),
        )
    }

    pub fn or_is_null(&mut self, column: &str) -> &mut Self {
        self.where_operator(
            column,
            Operator::Null,
            FieldValue::Null,
            Some(WhereJoin::Or),
        )
    }

    pub fn without_trash(&mut self) -> &mut Self {
        self.is_null(DELETED_AT_FIELD)
    }

    pub fn with_trash(&mut self) -> &mut Self {
        self.is_null(DELETED_AT_FIELD)
            .or_is_not_null(DELETED_AT_FIELD)
    }

    pub fn is_not_null(&mut self, column: &str) -> &mut Self {
        self.where_operator(column, Operator::NotNull, FieldValue::Null, None)
    }

    pub fn and_is_not_null(&mut self, column: &str) -> &mut Self {
        self.where_operator(
            column,
            Operator::NotNull,
            FieldValue::Null,
            Some(WhereJoin::And),
        )
    }

    pub fn or_is_not_null(&mut self, column: &str) -> &mut Self {
        self.where_operator(
            column,
            Operator::NotNull,
            FieldValue::Null,
            Some(WhereJoin::Or),
        )
    }

    pub fn is_in<T: Into<FieldValue> + IntoIterator>(
        &mut self,
        column: &str,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::In, value, None)
    }

    pub fn and_is_in<T: Into<FieldValue> + IntoIterator>(
        &mut self,
        column: &str,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::In, value, Some(WhereJoin::And))
    }

    pub fn or_is_in<T: Into<FieldValue> + IntoIterator>(
        &mut self,
        column: &str,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::In, value, Some(WhereJoin::Or))
    }

    pub fn is_not_in<T: Into<FieldValue> + IntoIterator>(
        &mut self,
        column: &str,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::NotIn, value, None)
    }

    pub fn and_is_not_in<T: Into<FieldValue> + IntoIterator>(
        &mut self,
        column: &str,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::NotIn, value, Some(WhereJoin::And))
    }

    pub fn or_is_not_in<T: Into<FieldValue> + IntoIterator>(
        &mut self,
        column: &str,
        value: T,
    ) -> &mut Self {
        self.where_operator(column, Operator::NotIn, value, Some(WhereJoin::Or))
    }

    pub fn between<T: Into<FieldValue>>(&mut self, column: &str, first: T, last: T) -> &mut Self {
        self.gt_or_eq(column, first).and_le_or_eq(column, last)
    }

    pub fn and_between<T: Into<FieldValue>>(
        &mut self,
        column: &str,
        first: T,
        last: T,
    ) -> &mut Self {
        self.and_gt_or_eq(column, first).and_le_or_eq(column, last)
    }

    pub fn or_between<T: Into<FieldValue>>(
        &mut self,
        column: &str,
        first: T,
        last: T,
    ) -> &mut Self {
        self.or_gt_or_eq(column, first).and_le_or_eq(column, last)
    }

    pub fn not_between<T: Into<FieldValue>>(
        &mut self,
        column: &str,
        first: T,
        last: T,
    ) -> &mut Self {
        self.not_gt_or_eq(column, first)
            .and_not_le_or_eq(column, last)
    }

    pub fn and_not_between<T: Into<FieldValue>>(
        &mut self,
        column: &str,
        first: T,
        last: T,
    ) -> &mut Self {
        self.and_not_gt_or_eq(column, first)
            .and_not_le_or_eq(column, last)
    }

    pub fn or_not_between<T: Into<FieldValue>>(
        &mut self,
        column: &str,
        first: T,
        last: T,
    ) -> &mut Self {
        self.or_not_gt_or_eq(column, first)
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

    pub fn where_operator<T: Into<FieldValue>>(
        &mut self,
        column: &str,
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

    fn init_select_columns_vec(&mut self) {
        if self.select_columns.is_none() {
            self.select_columns = Some(Vec::new());
            self.all_columns = false;
        }
    }
}
