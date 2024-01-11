use anyhow::anyhow;
use async_trait::async_trait;
use futures::stream::TryStreamExt;
use sqlx::{mysql::MySqlRow, types::chrono, Column, MySql, Pool, Row};
use std::{collections::HashMap, sync::Arc};

use crate::{
    base::{
        column::{BaseColumn, ColumnDefault, ColumnType},
        query::{QueryAction, QueryBuilder},
        query_conditions::Condition,
        query_operators::Operator,
        schema::{DatabaseKind, RelationalDbTrait, SchemaManagerTrait},
        table::BaseTable,
    },
    field_values::FieldValue,
    query_values::QueryValue,
    types::ColumnAndValue,
};

#[derive(Debug, Clone)]
struct ActiveQuery {
    statement: String,
    params: Vec<String>,
}

const LOG_TARGET: &str = "mysql_db_driver";

impl ActiveQuery {
    fn to_sql_string(&self) -> String {
        let mut query = self.statement.clone();
        for a_param in &self.params {
            query = query.replacen('?', a_param, 1);
        }

        query
    }
}
pub struct MySqlSchemaManager {
    db_pool: Arc<Pool<MySql>>,
}

impl MySqlSchemaManager {
    pub fn new(db_pool: Arc<Pool<MySql>>) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl RelationalDbTrait for MySqlSchemaManager {
    fn kind(&self) -> DatabaseKind {
        DatabaseKind::Mysql
    }
}

#[async_trait]
impl SchemaManagerTrait for MySqlSchemaManager {
    fn fetch_table_for_update(&self, name: &str) -> BaseTable {
        BaseTable::new(name)
    }
    async fn has_table(&self, name: &str) -> bool {
        let query = "SELECT table_name FROM INFORMATION_SCHEMA.TABLES WHERE table_name = ? AND table_schema = ?";

        let database = self
            .db_pool
            .connect_options()
            .as_ref()
            .get_database()
            .unwrap()
            .to_string();

        let result = sqlx::query(query)
            .bind(name)
            .bind(database)
            .map(|_| true)
            .fetch_one(self.db_pool.as_ref())
            .await;

        result.unwrap_or(false)
    }

    async fn stream_result(
        &self,
        query_builder: &QueryBuilder,
        sender: tokio::sync::mpsc::Sender<ColumnAndValue>,
    ) {
        let mut params = Vec::new();
        let statement = self.build_query(query_builder, &mut params);

        let mut query = sqlx::query(&statement);
        for p in &params {
            query = query.bind::<&str>(p);
        }

        let mut rows = query.fetch(self.db_pool.as_ref());
        while let Ok(result) = rows.try_next().await {
            if let Some(row) = result {
                if let Err(e) = sender.send(self.row_to_column_value(&row)).await {
                    log::error!(target: LOG_TARGET, "could not send mpsc stream: {}", e.to_string());
                }
            } else {
                break;
            }
        }
    }

    async fn drop_table(&self, name: &str) -> bool {
        if self.has_table(name).await {
            let query = QueryBuilder::new(vec![name.to_string()], QueryAction::DropTable);
            self.do_execute(query).await;
            return true;
        }

        false
    }

    async fn apply(&self, table: BaseTable) {
        self.do_apply(table).await
    }

    async fn execute(&self, query: QueryBuilder) {
        self.do_execute(query).await
    }

    async fn fetch_all(
        &self,
        query_builder: &QueryBuilder,
    ) -> Result<Option<Vec<HashMap<String, FieldValue>>>, anyhow::Error> {
        let mut results = Vec::new();

        let mut params = Vec::new();
        let statement = self.build_query(query_builder, &mut params);

        let mut query = sqlx::query(&statement);
        for p in &params {
            query = query.bind::<&str>(p);
        }

        let mut rows = query.fetch(self.db_pool.as_ref());

        loop {
            let next = rows.try_next().await;
            match next {
                Ok(result) => {
                    if let Some(row) = result {
                        results.push(self.row_to_column_value(&row));
                    } else {
                        break;
                    }
                }
                Err(e) => {
                    return Err(anyhow!("could not fetch rows: {}", e));
                }
            }
        }

        Ok(Some(results))
    }

    async fn fetch_one(
        &self,
        query_builder: &QueryBuilder,
    ) -> Result<Option<ColumnAndValue>, anyhow::Error> {
        let mut params = Vec::new();
        let statement = self.build_query(query_builder, &mut params);

        let mut query = sqlx::query(&statement);
        for p in &params {
            query = query.bind::<&str>(p);
        }
        return match query.fetch_optional(self.db_pool.as_ref()).await {
            Ok(result) => match result {
                Some(row) => Ok(Some(self.row_to_column_value(&row))),
                None => Ok(None),
            },
            Err(e) => Err(e.into()),
        };
    }

    async fn raw_insert(&self, statement: &str, args: Vec<String>) {
        let mut query = sqlx::query(statement);
        for p in args {
            query = query.bind(p);
        }
        let result = query.execute(self.db_pool.as_ref()).await;
        dbg!(result);
        dbg!(statement);
    }
}

impl MySqlSchemaManager {
    async fn do_apply(&self, table: BaseTable) {
        if table.view_query.is_some() {
            // working with view table
            self.create_or_replace_view(table).await
        } else {
            // working with real table
            self.apply_table_changes(table).await
        }
    }

    async fn do_execute(&self, query: QueryBuilder) {
        let mut params = Vec::new();

        let mut sql;
        match query.action() {
            QueryAction::Create {
                rows,
                do_soft_insert,
            } => {
                sql = format!(
                    "INSERT {} INTO {} ",
                    if *do_soft_insert { "IGNORE" } else { "" },
                    query.tables().join(",")
                );

                if !rows.is_empty() {
                    let keys = rows
                        .first()
                        .unwrap()
                        .keys()
                        .cloned()
                        .collect::<Vec<String>>();

                    let placeholders = keys.iter().map(|_| "?").collect::<Vec<&str>>().join(",");
                    let columns = keys
                        .iter()
                        .map(|e| format!("`{}`", e))
                        .collect::<Vec<String>>()
                        .join(",");

                    sql = format!("{} ({}) VALUES ", sql, columns);

                    for a_row in rows.iter().enumerate() {
                        let values = keys.iter().map(|col| {
                            let field = a_row.1.get(col).unwrap();
                            self.field_value_to_string(field)
                        });
                        let separator = if a_row.0 > 0 { "," } else { "" };

                        params.extend(values);
                        sql = format!("{} {} ({})", sql, separator, &placeholders);
                    }
                }
            }
            QueryAction::Update(column_values) => {
                let mut columns = Vec::new();
                for entry in column_values {
                    if *entry.1 != FieldValue::NotSet {
                        columns.push(entry.0);
                        params.push(self.field_value_to_string(entry.1));
                    }
                }
                sql = format!("UPDATE `{}` SET ", query.tables().join(","));
                for entry in columns.iter().enumerate() {
                    if entry.0 > 0 {
                        sql = format!("{}, `{}` = ? ", sql, *entry.1);
                    } else {
                        sql = format!("{} `{}` = ? ", sql, *entry.1);
                    }
                }

                // joins
                sql = format!("{} {}", sql, self.build_join(&query, &mut params));
                // where
                sql = format!("{} {}", sql, self.build_where_clauses(&query, &mut params));
            }
            QueryAction::Delete => {
                sql = format!("DELETE {0} FROM {0} ", query.tables().join(","));
                // joins
                sql = format!("{} {}", sql, self.build_join(&query, &mut params));
                // where
                sql = format!("{} {}", sql, self.build_where_clauses(&query, &mut params));
            }
            QueryAction::DropTable => {
                sql = query
                    .tables()
                    .iter()
                    .map(|name| format!("DROP TABLE {};", name))
                    .fold(String::new(), |sum, sub| format!("{} {}", sum, sub));
            }
            _ => {
                sql = "".into();
            }
        }

        let mut query_statement = sqlx::query(&sql);

        for p in &params {
            query_statement = query_statement.bind(p);
        }

        let result = query_statement.execute(self.db_pool.as_ref()).await;

        match result {
            Ok(r) => {
                log::debug!("{} result: {:#?}", query.action(), r);
            }
            Err(e) => {
                log::debug!("{} failed: {}", query.action(), e);
            }
        }
    }

    async fn create_or_replace_view(&self, table: BaseTable) {
        if let Some(query) = &table.view_query {
            let mut params = Vec::new();
            let sql = self.build_query(query, &mut params);

            let active_query = ActiveQuery {
                statement: sql,
                params,
            };

            let query = format!(
                "CREATE OR REPLACE VIEW `{}` AS ({})",
                &table.name,
                active_query.to_sql_string()
            );

            let result = sqlx::query(&query).execute(self.db_pool.as_ref()).await;
            match result {
                Ok(_) => {
                    log::info!("View '{}' created or replaced successfully", &table.name);
                }
                Err(error) => {
                    log::error!(
                        "Could not create or replace view '{}': {:#?}",
                        &table.name,
                        error
                    );
                }
            }
        }
    }

    async fn apply_table_changes(&self, table: BaseTable) {
        let columns: Vec<String> = table
            .columns()
            .iter()
            .map(|column| self.create_column(column))
            .collect();

        let mut query = if table.is_new() {
            format!("CREATE TABLE `{}`", &table.name)
        } else {
            format!("ALTER TABLE `{}`", &table.name)
        };

        if !columns.is_empty() {
            query = format!("{} ({})", query, columns.join(","));
        }

        query = format!("{} ENGINE='InnoDB';", query);

        let result = sqlx::query(&query).execute(self.db_pool.as_ref()).await;

        match result {
            Ok(_) => {
                log::info!(
                    "Table '{}' {} successfully",
                    &table.name,
                    if table.is_new() { "created" } else { "updated" }
                );
            }
            Err(e) => {
                let name;
                let action;

                if table.is_new() {
                    action = "create";
                    name = table.new_name.unwrap_or(table.name.clone())
                } else {
                    action = "update";
                    name = table.name.clone();
                }
                log::error!("Could not {} table {}: {}", action, name, e);
            }
        }

        // create/update indexes
        if let Some(indexes) = &table.indexes {
            for entry in indexes {
                let sql = format!("ALTER TABLE {} {}", &table.name, entry);

                let index_result = sqlx::query(&sql).execute(self.db_pool.as_ref()).await;
                match index_result {
                    Ok(_e) => log::info!("table index created"),
                    Err(e) => {
                        log::error!("could not create table index: {}", e.to_string())
                    }
                }
            }
        }
    }

    fn create_column(&self, column: &BaseColumn) -> String {
        let mut entry = format!("`{}`", &column.name);
        let mut the_type = " ".to_owned();

        // column type
        match column.column_type {
            ColumnType::AutoIncrementId => {
                the_type.push_str("bigint(20) AUTO_INCREMENT PRIMARY KEY")
            }
            ColumnType::Boolean => the_type.push_str("tinyint(1)"),
            ColumnType::Char(length) => {
                the_type.push_str(&format!("char({}) COLLATE 'utf8mb4_unicode_ci'", length))
            }
            ColumnType::Datetime => the_type.push_str("datetime"),
            ColumnType::Timestamp => the_type.push_str("timestamp"),
            // ColumnType::File() shouldn't be here
            // ColumnType::Float not sure
            ColumnType::Integer => the_type.push_str("bigint(20)"),
            ColumnType::Json => the_type.push_str("json"),
            ColumnType::Number => the_type.push_str("double"),
            // ColumnType::Relation { relation_type, table_name }
            // ColumnType::Select()
            ColumnType::String(length) => {
                let q = format!("varchar({}) COLLATE 'utf8mb4_unicode_ci'", length);
                the_type.push_str(q.as_str());
            }
            ColumnType::Text => the_type.push_str("longtext"),
            ColumnType::Uuid => the_type.push_str("uuid"),
            _ => the_type.push_str("varchar(255)"),
        };

        // column is nullable
        if let Some(nullable) = column.is_nullable {
            if nullable {
                the_type.push_str(" NULL");
            } else {
                the_type.push_str(" NOT NULL");
            }
        }

        // column is unique
        if column.is_unique {
            the_type.push_str(" UNIQUE");
        }

        // column default
        if let Some(default) = &column.default {
            the_type.push_str(" DEFAULT ");
            match default {
                ColumnDefault::CreatedAt => the_type.push_str("now()"),
                ColumnDefault::Custom(d) => the_type.push_str(&format!("'{}'", d)),
                ColumnDefault::EmptyArray => the_type.push_str("[]"),
                ColumnDefault::EmptyObject => the_type.push_str("{}"),
                ColumnDefault::EmptyString => the_type.push_str(""),
                ColumnDefault::Uuid => the_type.push_str("SYS_GUID()"),
                ColumnDefault::Ulid => (),
                ColumnDefault::UpdatedAt => {
                    the_type.push_str("current_timestamp() ON UPDATE CURRENT_TIMESTAMP")
                }
                ColumnDefault::Zero => the_type.push('0'),
            };
        }

        // column relationship
        if let Some(relationship) = &column.relationship {
            the_type.push_str(&format!(
                ", FOREIGN KEY (`{}`) REFERENCES `{}` (`{}`)",
                &column.name,
                &relationship.table(),
                &relationship.column()
            ));
            if relationship.cascade_delete() {
                the_type.push_str(" ON DELETE CASCADE");
            }
        }

        entry.push_str(&the_type);
        entry
    }

    fn build_query(&self, query: &QueryBuilder, params: &mut Vec<String>) -> String {
        let mut sql = "SELECT".to_owned();

        // fields
        if let QueryAction::Query {
            columns,
            select_all,
        } = query.action()
        {
            if *select_all {
                sql = format!("{} *", sql) // Select all columns by default
            } else if let Some(fields) = columns {
                sql = format!("{} {}", sql, fields.join(","));
            }
        }

        // join fields
        if let Some(joins) = query.joins() {
            for a_join in joins {
                match a_join.select_columns() {
                    Some(columns) => {
                        sql = format!("{}, {}", sql, columns.join(","));
                    }
                    None => (),
                }
            }
        }

        // from
        sql = format!("{} FROM {}", sql, query.tables().join(","));

        // joins
        sql = format!("{} {}", sql, self.build_join(query, params));

        // wheres
        sql = format!("{} {}", sql, self.build_where_clauses(query, params));

        // group by

        // order by
        if let Some(order) = self.build_order_by(query) {
            sql = format!("{} {}", sql, order);
        }

        // having

        // limit, offset
        dbg!("----> {}", &sql);

        sql
    }

    fn build_join(&self, query: &QueryBuilder, _params: &mut [String]) -> String {
        let mut sql = "".to_string();
        if let Some(joins) = query.joins() {
            for a_join in joins {
                sql = format!(
                    "{} {} JOIN {} ON {}",
                    sql,
                    a_join.join_type(),
                    a_join.table(),
                    a_join.join_clause()
                );
            }
        }

        sql
    }

    fn build_where_clauses(&self, query: &QueryBuilder, params: &mut Vec<String>) -> String {
        let mut wheres = "".to_owned();
        for where_join in query.where_clauses() {
            wheres = where_join.as_clause(
                &wheres,
                &self.transform_condition(where_join.condition(), params),
            );
        }

        if !wheres.is_empty() {
            wheres = format!("WHERE {}", wheres);
        }

        wheres
    }

    fn build_order_by(&self, query: &QueryBuilder) -> Option<String> {
        match query.order_by() {
            Some(order) => Some(order.to_string()),
            _ => None,
        }
    }

    fn transform_condition(&self, condition: &Condition, params: &mut Vec<String>) -> String {
        self.transform_value(condition.value(), params);

        let placeholder =
            if *condition.operator() == Operator::In || *condition.operator() == Operator::NotIn {
                let length = match &condition.value() {
                    QueryValue::Field(FieldValue::Array(v)) => v.len(),
                    _ => 1,
                };

                let mut placeholder = Vec::new();
                placeholder.resize(length, "?");
                placeholder.join(",")
            } else {
                "?".to_owned()
            };

        condition
            .operator()
            .as_clause(condition.column(), &placeholder)
    }

    fn transform_value(&self, value: &QueryValue, params: &mut Vec<String>) {
        match value {
            QueryValue::SubQuery(q) => {
                self.build_query(q, params);
            }
            _ => value.to_param(params),
        }
    }

    fn row_to_column_value(&self, row: &MySqlRow) -> ColumnAndValue {
        let mut this_row = HashMap::new();

        for col in row.columns() {
            let name = col.name().to_string();
            match col.type_info().to_string().as_str() {
                "BOOLEAN" | "TINYINT(1)" => {
                    let v: bool = row.try_get::<i8, &str>(col.name()).unwrap_or_default() > 0;
                    this_row.insert(name, FieldValue::Boolean(v));
                }
                "TINYINT" => {
                    let v = row.try_get::<i8, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, (v as i32).into());
                    } else {
                        this_row.insert(name, 0_i32.into());
                    }
                }
                "SMALLINT" => {
                    let v = row.try_get::<i16, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, (v as i32).into());
                    } else {
                        this_row.insert(name, 0_i32.into());
                    }
                }
                "INT" => {
                    let v = row.try_get::<i32, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, v.into());
                    } else {
                        this_row.insert(name, 0_i32.into());
                    }
                }
                "BIGINT" => {
                    let v = row.try_get::<i64, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, v.into());
                    } else {
                        this_row.insert(name, 0_i64.into());
                    }
                }
                "TINYINT UNSIGNED" => {
                    let v = row.try_get::<u8, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, (v as u32).into());
                    } else {
                        this_row.insert(name, 0_u32.into());
                    }
                }
                "SMALLINT UNSIGNED" => {
                    let v = row.try_get::<u16, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, (v as u32).into());
                    } else {
                        this_row.insert(name, 0_u32.into());
                    }
                }
                "INT UNSIGNED" => {
                    let v = row.try_get::<u32, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, v.into());
                    } else {
                        this_row.insert(name, 0_u32.into());
                    }
                }
                "BIGINT UNSIGNED" => {
                    let v = row.try_get::<u64, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, v.into());
                    } else {
                        this_row.insert(name, 0_u64.into());
                    }
                }
                "DOUBLE" | "FLOAT" => {
                    let v = row.try_get::<f64, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, v.into());
                    } else {
                        this_row.insert(name, 0.0_f64.into());
                    }
                }
                "CHAR" | "VARCHAR" | "TEXT" => {
                    if let Ok(v) = row.try_get::<String, &str>(col.name()) {
                        this_row.insert(name, v.into());
                    } else {
                        this_row.insert(name, FieldValue::Null);
                    }
                }
                "TIMESTAMP" => {
                    let v = row.try_get::<chrono::DateTime<chrono::Utc>, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, FieldValue::Timestamp(v));
                    } else {
                        this_row.insert(name, FieldValue::Null);
                    }
                }
                "DATE" => {
                    let v = row.try_get::<chrono::NaiveDate, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, FieldValue::Date(v));
                    } else {
                        this_row.insert(name, FieldValue::Null);
                    }
                }
                "TIME" => {
                    let v = row.try_get::<chrono::NaiveTime, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, v.to_string().into());
                    } else {
                        this_row.insert(name, FieldValue::Null);
                    }
                }
                "DATETIME" => {
                    let v = row.try_get::<chrono::NaiveDateTime, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(col.name().to_owned(), FieldValue::DateTime(v.and_utc()));
                    } else {
                        this_row.insert(col.name().to_owned(), FieldValue::Null);
                    }
                }
                "VARBINARY" | "BINARY" | "BLOB" => {}
                // TODO find a means to represent binary
                _ => {
                    dbg!(
                        "not mapped field: {:#?} => value: {:#?}",
                        name,
                        col.type_info()
                    );
                }
            }
        }
        this_row
    }

    fn field_value_to_string(&self, field: &FieldValue) -> String {
        match field {
            FieldValue::DateTime(dt) => {
                format!("{}", dt.format("%F %T"))
            }
            FieldValue::Timestamp(dt) => {
                format!("{}", dt.format("%F %T"))
            }
            _ => field.to_string(),
        }
    }
}

// enum MyRow<'a> {
//     MySql(&'a MySqlRow),
// }
// fn parse_row(row: MyRow) {
//     match row {
//         MyRow::MySql(row) => {
//             for col in row.columns() {
//                 dbg!(col.name(), row.try_get::<Option<String>, &str>(col.name()));
//             }
//         }
//     }
// }
