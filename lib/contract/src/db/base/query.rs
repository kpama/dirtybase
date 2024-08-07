use crate::db::{
    field_values::FieldValue,
    query_values::QueryValue,
    types::{ColumnAndValue, FromColumnAndValue, StructuredColumnAndValue},
    TableEntityTrait,
};

use super::{
    aggregate::Aggregate,
    column::BaseColumn,
    join_builder::JoinQueryBuilder,
    order_by_builder::{LimitBuilder, OffsetBuilder, OrderByBuilder},
    query_conditions::Condition,
    query_join_types::JoinType,
    query_operators::Operator,
    schema::SchemaManagerTrait,
    table::DELETED_AT_FIELD,
    where_join_operators::WhereJoinOperator,
};
use std::{collections::HashMap, fmt::Display, marker::PhantomData};

#[derive(Debug)]
pub enum WhereJoin {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryAction {
    Query {
        columns: Option<Vec<String>>,
    },
    Create {
        rows: Vec<ColumnAndValue>,
        do_soft_insert: bool,
    },
    Update(HashMap<String, FieldValue>),
    Delete,
    DropTable,
    RenameTable(String),
    DropColumn(String),
    AddColumn(BaseColumn),
    RenameColumn {
        old: String,
        new: String,
    },
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
                QueryAction::Query { columns: _ } => "Query",
                QueryAction::Update(_) => "Update",
                QueryAction::Delete => "Delete",
                QueryAction::DropTable => "DropTable",
                QueryAction::RenameTable(_) => "RenameTable",
                QueryAction::DropColumn(_) => "DropColumn",
                QueryAction::AddColumn(_) => "AddColumn",
                QueryAction::RenameColumn { old: _, new: _ } => "RenameColumn",
            }
        )
    }
}

#[derive(Clone, Debug)]
pub struct QueryBuilder {
    where_clauses: Vec<WhereJoinOperator>,
    table: String,
    joins: Option<Vec<JoinQueryBuilder>>,
    action: QueryAction,
    order_by: Option<OrderByBuilder>,
    limit: Option<LimitBuilder>,
    offset: Option<OffsetBuilder>,
}

impl QueryBuilder {
    pub fn new(table: &str, action: QueryAction) -> Self {
        Self {
            where_clauses: Vec::new(),
            table: table.to_string(),
            joins: None,
            action,
            order_by: None,
            limit: None,
            offset: None,
        }
    }

    pub fn action(&self) -> &QueryAction {
        &self.action
    }

    pub fn table(&self) -> &String {
        &self.table
    }

    pub fn all_columns(&self) -> bool {
        match &self.action {
            QueryAction::Query { columns } => columns.is_some(),
            _ => false,
        }
    }

    pub fn sub_query<F>(&mut self, table: &str, mut callback: F) -> QueryValue
    where
        F: FnMut(&mut QueryBuilder),
    {
        let mut query_builder = Self::new(table, QueryAction::Query { columns: None });

        callback(&mut query_builder);

        QueryValue::SubQuery(query_builder)
    }

    /// By default all fields are select
    #[deprecated]
    pub fn select_all(&mut self) -> &mut Self {
        if let QueryAction::Query { columns: None } = &mut self.action {}

        self
    }

    pub fn count(&mut self, column: &str) -> &mut Self {
        let as_name = format!("count_{}", column.to_string());
        self.count_as(column, &as_name)
    }

    pub fn count_as(&mut self, column: &str, as_name: &str) -> &mut Self {
        self.select(format!(
            "{} as '{}'",
            Aggregate::Count(column.to_string()),
            as_name.to_string()
        ));
        self
    }

    pub fn max(&mut self, column: &str) -> &mut Self {
        let as_name = format!("max_{}", column.to_string());
        self.max_as(column, &as_name)
    }

    pub fn max_as(&mut self, column: &str, as_name: &str) -> &mut Self {
        self.select(format!(
            "{} as '{}'",
            Aggregate::Max(column.to_string()),
            as_name.to_string()
        ));
        self
    }

    pub fn min<C: ToString>(&mut self, column: C) -> &mut Self {
        let as_name = format!("min_{}", column.to_string());
        self.min_as(column, as_name)
    }

    pub fn min_as<C: ToString, A: ToString>(&mut self, column: C, as_name: A) -> &mut Self {
        self.select(format!(
            "{} as '{}'",
            Aggregate::Min(column.to_string()),
            as_name.to_string()
        ));
        self
    }
    pub fn sum<C: ToString>(&mut self, column: C) -> &mut Self {
        let as_name = format!("sum_{}", column.to_string());
        self.sum_as(column, as_name)
    }

    pub fn sum_as<C: ToString, A: ToString>(&mut self, column: C, as_name: A) -> &mut Self {
        self.select(format!(
            "{} as '{}'",
            Aggregate::Sum(column.to_string()),
            as_name.to_string()
        ));
        self
    }

    pub fn avg<C: ToString>(&mut self, column: C) -> &mut Self {
        let as_name = format!("avg_{}", column.to_string());
        self.avg_as(column, as_name)
    }

    pub fn avg_as<C: ToString, A: ToString>(&mut self, column: C, as_name: A) -> &mut Self {
        self.select(format!(
            "{} as '{}'",
            Aggregate::Avg(column.to_string()),
            as_name.to_string()
        ));
        self
    }

    pub fn joins(&self) -> Option<&Vec<JoinQueryBuilder>> {
        self.joins.as_ref()
    }

    pub fn order_by(&self) -> Option<&OrderByBuilder> {
        self.order_by.as_ref()
    }

    /// Set a column/value for update
    pub fn set_column<T: Into<FieldValue>, C: ToString>(
        &mut self,
        column: C,
        value: T,
    ) -> &mut Self {
        if let QueryAction::Update(columns) = &mut self.action {
            columns.insert(column.to_string(), value.into());
        }

        self
    }

    /// Set multiple column/value for update
    pub fn set_columns(&mut self, column_and_values: ColumnAndValue) -> &mut Self {
        if let QueryAction::Update(columns) = &mut self.action {
            columns.extend(column_and_values);
        }

        self
    }

    /// Set multiple rows for insert/create
    pub fn set_insert_rows(&mut self, rows_to_insert: Vec<ColumnAndValue>) -> &mut Self {
        if let QueryAction::Create {
            rows,
            do_soft_insert: _,
        } = &mut self.action
        {
            *rows = rows_to_insert;
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
        if let QueryAction::Query { columns } = &mut self.action {
            if let Some(list) = columns {
                list.push(column.to_string())
            } else {
                *columns = Some(vec![column.to_string()]);
            }
        }

        self
    }

    /// Adds a table to the list of tables to select from
    pub fn select_table<T: TableEntityTrait>(&mut self) -> &mut Self {
        self.select_multiple(&T::table_column_full_names())
    }

    /// Adds multiple columns to be selected
    pub fn select_multiple<T: ToString, C: IntoIterator<Item = T>>(
        &mut self,
        columns_to_select: C,
    ) -> &mut Self {
        if let QueryAction::Query { columns } = &mut self.action {
            if let Some(list) = columns {
                list.extend(columns_to_select.into_iter().map(|c| c.to_string()))
            } else {
                *columns = Some(
                    columns_to_select
                        .into_iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<String>>(),
                );
            }
        }

        self
    }

    pub fn asc<C: ToString>(&mut self, column: C) -> &mut Self {
        if self.order_by.is_none() {
            self.order_by = Some(OrderByBuilder::new());
        }

        self.order_by.as_mut().unwrap().asc(column);

        self
    }

    pub fn desc<C: ToString>(&mut self, column: C) -> &mut Self {
        if self.order_by.is_none() {
            self.order_by = Some(OrderByBuilder::new());
        }

        self.order_by.as_mut().unwrap().desc(column);

        self
    }

    pub fn limit(&mut self, limit: usize) -> &mut Self {
        self.limit = Some(LimitBuilder { limit });
        self
    }

    pub fn limit_by(&self) -> Option<LimitBuilder> {
        self.limit.clone()
    }

    pub fn offset(&mut self, offset: usize) -> &mut Self {
        self.offset = Some(OffsetBuilder { offset });
        self
    }

    pub fn offset_by(&self) -> Option<OffsetBuilder> {
        self.offset.clone()
    }

    pub fn and(&mut self, _query: Self) -> &mut Self {
        // TODO: Implement "and" group. example `where age = 3 and (....)`
        self
    }

    pub fn or(&mut self, _query: Self) -> &mut Self {
        // TODO: Implement "and" group. example `where age = 3 or (....)`
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

    // pub fn with_creator<L: TableEntityTrait>(&mut self, prefix: Option<&str>) -> &mut Self {
    //     if let Some(field) = L::creator_id_column() {
    //         self.left_join_table_and_select::<UserEntity, L>(
    //             UserEntity::id_column().unwrap(),
    //             &L::prefix_with_tbl(field),
    //             prefix,
    //         );
    //     }
    //     self
    // }

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
        left_field: impl ToString,
        right_field: impl ToString,
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

pub struct EntityQueryBuilder<T: FromColumnAndValue + Send + Sync + 'static> {
    query_builder: QueryBuilder,
    inner: Box<dyn SchemaManagerTrait>,
    phathom: PhantomData<T>,
}

impl<T: FromColumnAndValue + Send + Sync + 'static> EntityQueryBuilder<T> {
    pub fn new(query_builder: QueryBuilder, inner: Box<dyn SchemaManagerTrait>) -> Self {
        Self {
            query_builder,
            inner,
            phathom: PhantomData,
        }
    }

    pub fn query(&mut self) -> &mut QueryBuilder {
        &mut self.query_builder
    }

    pub async fn latest(mut self, column: &str) -> Result<Option<T>, anyhow::Error> {
        self.query_builder.desc(column);
        self.one().await
    }

    pub async fn oldest(mut self, column: &str) -> Result<Option<T>, anyhow::Error> {
        self.query_builder.asc(column);
        self.one().await
    }

    pub async fn count(mut self) -> Result<i64, anyhow::Error> {
        self.query().count_as("*", "count_all");
        match self.fetch_one().await {
            Ok(Some(r)) => Ok(i64::from(r.get("count_all").unwrap())),
            Ok(None) => Ok(0),
            Err(e) => Err(e),
        }
    }

    pub async fn all(self) -> Result<Option<Vec<T>>, anyhow::Error> {
        let result = self.fetch_all().await;
        if let Ok(records) = result {
            match records {
                Some(rows) => Ok(Some(
                    rows.into_iter()
                        .map(|row| T::from_column_value(row.fields()))
                        .collect::<Vec<T>>(),
                )),
                None => Ok(Some(Vec::new())),
            }
        } else {
            Err(result.err().unwrap())
        }
    }

    pub async fn one(self) -> Result<Option<T>, anyhow::Error> {
        let result = self.fetch_one().await;

        if let Ok(row) = result {
            match row {
                Some(r) => Ok(Some(T::from_column_value(r.fields()))),
                None => Ok(None),
            }
        } else {
            Err(result.err().unwrap())
        }
    }

    pub async fn stream(self) -> tokio_stream::wrappers::ReceiverStream<T> {
        let (inner_sender, mut inner_receiver) = tokio::sync::mpsc::channel::<ColumnAndValue>(100);
        let (outer_sender, outer_receiver) = tokio::sync::mpsc::channel::<T>(100);

        tokio::spawn(async move {
            while let Some(result) = inner_receiver.recv().await {
                if let Err(e) = outer_sender.send(T::from_column_value(result)).await {
                    log::debug!("error sending transformed row result: {}", e);
                    break;
                }
            }
        });

        tokio::spawn(async move {
            self.inner
                .stream_result(&self.query_builder, inner_sender)
                .await;
        });

        tokio_stream::wrappers::ReceiverStream::new(outer_receiver)
    }

    async fn fetch_all(&self) -> Result<Option<Vec<StructuredColumnAndValue>>, anyhow::Error> {
        let results = self.inner.fetch_all(&self.query_builder).await;
        if let Ok(records) = results {
            match records {
                Some(rs) => Ok(Some(StructuredColumnAndValue::from_results(rs))),
                None => Ok(Some(Vec::new())),
            }
        } else {
            Err(results.err().unwrap())
        }
    }

    async fn fetch_one(&self) -> Result<Option<StructuredColumnAndValue>, anyhow::Error> {
        let result = self.inner.fetch_one(&self.query_builder).await;

        if let Ok(row) = result {
            match row {
                Some(r) => Ok(Some(StructuredColumnAndValue::from_a_result(r))),
                None => Ok(None),
            }
        } else {
            Err(result.err().unwrap())
        }
    }
}
