use crate::base::{
    column::{BaseColumn, ColumnDefault, ColumnType},
    field_values::FieldValue,
    index::IndexType,
    query::{QueryAction, QueryBuilder},
    query_conditions::Condition,
    query_operators::Operator,
    query_values::QueryValue,
    schema::{RelationalDbTrait, SchemaManagerTrait},
    table::{BaseTable, UPDATED_AT_FIELD},
    types::ColumnAndValue,
};
use async_trait::async_trait;
use futures::stream::TryStreamExt;
use sqlx::{any::AnyKind, sqlite::SqliteRow, types::chrono, Column, Pool, Row, Sqlite};
use std::{collections::HashMap, sync::Arc};

struct ActiveQuery {
    statement: String,
    params: Vec<String>,
}

impl ActiveQuery {
    fn to_sql_string(&self) -> String {
        let mut query = self.statement.clone();
        for a_param in &self.params {
            query = query.replacen('?', a_param, 1);
        }

        query
    }
}
pub struct SqliteSchemaManager {
    db_pool: Arc<Pool<Sqlite>>,
    active_query: Option<ActiveQuery>,
}

impl SqliteSchemaManager {
    pub fn new(db_pool: Arc<Pool<Sqlite>>) -> Self {
        Self {
            db_pool,
            active_query: None,
        }
    }
}

#[async_trait]
impl RelationalDbTrait for SqliteSchemaManager {
    fn kind(&self) -> AnyKind {
        AnyKind::Sqlite
    }
}

#[async_trait]
impl SchemaManagerTrait for SqliteSchemaManager {
    fn fetch_table_for_update(&self, name: &str) -> BaseTable {
        BaseTable::new(name)
    }
    async fn has_table(&self, name: &str) -> bool {
        let query = "SELECT name FROM sqlite_master WHERE name = ?";

        let result = sqlx::query(query)
            .bind(name)
            .map(|_row| true)
            .fetch_one(self.db_pool.as_ref())
            .await;

        result.unwrap_or(false)
    }

    async fn commit(&self, table: BaseTable) {
        self.do_commit(table).await
    }

    fn query(&mut self, query: QueryBuilder) -> &dyn SchemaManagerTrait
    where
        Self: Sized,
    {
        let mut params = Vec::new();
        let statement = self.build_query(&query, &mut params);

        self.active_query = Some(ActiveQuery { statement, params });

        self
    }

    async fn execute(&self, query: QueryBuilder) {
        self.do_execute(query).await
    }

    async fn fetch_all_as_json(&self) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let mut results = Vec::new();
        match &self.active_query {
            Some(active_query) => {
                let mut query = sqlx::query(&active_query.statement);
                for p in &active_query.params {
                    query = query.bind::<&str>(p);
                }

                let mut rows = query.fetch(self.db_pool.as_ref());
                while let Ok(result) = rows.try_next().await {
                    if let Some(row) = result {
                        results.push(self.row_to_json(&row));
                    }
                }
            }
            None => (),
        }

        Ok(results)
    }

    async fn fetch_one_as_json(&self) -> Result<serde_json::Value, anyhow::Error> {
        if let Some(active_query) = &self.active_query {
            let mut query = sqlx::query(&active_query.statement);
            for p in &active_query.params {
                query = query.bind::<&str>(p);
            }
            return match query.fetch_one(self.db_pool.as_ref()).await {
                Ok(row) => Ok(self.row_to_json(&row)),
                Err(e) => Err(e.into()),
            };
        }

        Err(anyhow::anyhow!("No query to execute"))
    }

    async fn fetch_all_as_field_value(
        &self,
    ) -> Result<Vec<HashMap<String, FieldValue>>, anyhow::Error> {
        let mut results = Vec::new();

        match &self.active_query {
            Some(active_query) => {
                let mut query = sqlx::query(&active_query.statement);
                for p in &active_query.params {
                    query = query.bind::<&str>(p);
                }

                let mut rows = query.fetch(self.db_pool.as_ref());
                while let Ok(result) = rows.try_next().await {
                    if let Some(row) = result {
                        results.push(self.row_to_insert_value(&row));
                    }
                }
            }
            None => (),
        }

        Ok(results)
    }

    async fn fetch_one_as_field_value(&self) -> Result<ColumnAndValue, anyhow::Error> {
        if let Some(active_query) = &self.active_query {
            let mut query = sqlx::query(&active_query.statement);
            for p in &active_query.params {
                query = query.bind::<&str>(p);
            }
            return match query.fetch_one(self.db_pool.as_ref()).await {
                Ok(row) => Ok(self.row_to_insert_value(&row)),
                Err(e) => Err(e.into()),
            };
        }

        Err(anyhow::anyhow!("No query to execute"))
    }
}

impl SqliteSchemaManager {
    async fn do_commit(&self, table: BaseTable) {
        if table.view_query.is_some() {
            // working with view table
            self.create_or_replace_view(table).await
        } else {
            // working with real table
            self.apply_table_changes(table).await
        }
    }

    async fn do_execute(&self, query: QueryBuilder) {
        let mut columns = Vec::new();
        let mut params = Vec::new();
        if let Some(list) = query.set_columns() {
            for entry in list {
                if *entry.1 != FieldValue::NotSet {
                    columns.push(entry.0);
                    params.push(entry.1.to_string());
                }
            }
        }

        let mut sql;
        match query.action() {
            QueryAction::Create => {
                sql = format!("INSERT INTO {} (", query.tables().join(","));
                for entry in columns.iter().enumerate() {
                    if entry.0 > 0 {
                        sql = format!("{}, `{}`", sql, *entry.1);
                    } else {
                        sql = format!("{} `{}`", sql, *entry.1);
                    }
                }
                sql = format!(
                    "{} ) VALUES ({})",
                    sql,
                    params.iter().map(|_| "?").collect::<Vec<&str>>().join(",")
                );
            }
            QueryAction::Update => {
                sql = format!("UPDATE `{}` SET ", query.tables().join(","));
                for entry in columns.iter().enumerate() {
                    if entry.0 > 0 {
                        sql = format!("{}, `{}` = ? ", sql, *entry.1);
                    } else {
                        sql = format!("{} `{}` = ? ", sql, *entry.1);
                    }
                }

                sql = format!("{} {}", sql, self.build_where_clauses(&query, &mut params));
            }
            QueryAction::Delete => {
                sql = format!("DELETE FROM {} (", query.tables().join(","));
                sql = format!("{} {}", sql, self.build_where_clauses(&query, &mut params));
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
        let mut foreigns = Vec::new();
        let columns: Vec<String> = table
            .columns()
            .iter()
            .map(|column| self.create_column(column, &mut foreigns))
            .collect();

        let mut query = if table.is_new() {
            format!("CREATE TABLE `{}`", &table.name)
        } else {
            format!("ALTER TABLE `{}`", &table.name)
        };

        if !columns.is_empty() {
            query = if foreigns.is_empty() {
                format!("{} ({})", query, columns.join(","))
            } else {
                format!("{} ({}, {})", query, columns.join(","), foreigns.join(","))
            }
        }

        // query = format!("{} ENGINE='InnoDB';", query);
        if query.contains(UPDATED_AT_FIELD) {
            query = format!(
                "{}; CREATE TRIGGER IF NOT EXISTS {1}_updated_at_trigger AFTER UPDATE ON {1} BEGIN UPDATE core_user SET {2} = CURRENT_TIMESTAMP; END; ",
                query, &table.name, UPDATED_AT_FIELD
            )
        }

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
                let sql;
                match entry {
                    IndexType::Index(index) | IndexType::Primary(index) => {
                        if index.delete_index() {
                            sql = format!("DROP INDEX IF EXISTS {}.{}", &table.name, index.name());
                        } else {
                            sql = format!(
                                "CREATE INDEX IF NOT EXISTS '{}' ON {} ({})",
                                index.name(),
                                &table.name,
                                index.concat_columns()
                            );
                        }
                    }
                    IndexType::Unique(index) => {
                        if index.delete_index() {
                            sql = format!("DROP INDEX IF EXISTS {}.{}", &table.name, index.name());
                        } else {
                            sql = format!(
                                "CREATE UNIQUE  INDEX IF NOT EXISTS '{}' ON {} ({})",
                                &table.name,
                                index.name(),
                                index.concat_columns()
                            );
                        }
                    }
                }

                let index_result = sqlx::query(&sql).execute(self.db_pool.as_ref()).await;
                match index_result {
                    Ok(_e) => log::info!("table index created"),
                    Err(e) => {
                        log::error!("sql: {}", &sql);
                        log::error!("could not create table index: {}", e.to_string())
                    }
                }
            }
        }
    }

    fn create_column(&self, column: &BaseColumn, foreigns: &mut Vec<String>) -> String {
        let mut entry = format!("`{}`", &column.name);
        let mut the_type = " ".to_owned();

        // column type
        match column.column_type {
            ColumnType::AutoIncrementId => {
                the_type.push_str("INTEGER");
                foreigns.push(format!("PRIMARY KEY('{}' AUTOINCREMENT)", &column.name));
            }
            ColumnType::Boolean => the_type.push_str("BOOLEAN"),
            ColumnType::Char(length) => the_type.push_str(&format!("VARCHAR({})", length)),
            ColumnType::Date => the_type.push_str("datetime"),
            // ColumnType::File() shouldn't be here
            // ColumnType::Float not sure
            ColumnType::Integer => the_type.push_str("INTEGER"),
            ColumnType::Json => the_type.push_str("json"),
            ColumnType::Number => the_type.push_str("double"),
            // ColumnType::Relation { relation_type, table_name }
            // ColumnType::Select()
            ColumnType::String(length) => {
                let q = format!("VARCHAR({})", length);
                the_type.push_str(q.as_str());
            }
            ColumnType::Text => the_type.push_str("TEXT"),
            ColumnType::Uuid => the_type.push_str("uuid"),
            _ => the_type.push_str("VARCHAR(255)"),
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
                ColumnDefault::CreatedAt => the_type.push_str("CURRENT_TIMESTAMP"),
                ColumnDefault::Custom(d) => the_type.push_str(&format!("'{}'", d)),
                ColumnDefault::EmptyArray => the_type.push_str("[]"),
                ColumnDefault::EmptyObject => the_type.push_str("{}"),
                ColumnDefault::EmptyString => the_type.push_str(""),
                ColumnDefault::Uuid => the_type.push_str("GUID()"),
                ColumnDefault::Ulid => (),
                ColumnDefault::UpdatedAt => {
                    the_type.push_str("CURRENT_TIMESTAMP")
                    // the_type.push_str("current_timestamp() ON UPDATE CURRENT_TIMESTAMP")
                }
                ColumnDefault::Zero => the_type.push('0'),
            };
        }

        // column relationship
        if let Some(relationship) = &column.relationship {
            let mut f = "".to_string();
            f.push_str(&format!(
                "FOREIGN KEY (`{}`) REFERENCES `{}` (`{}`)",
                &column.name,
                &relationship.table(),
                &relationship.column()
            ));
            if relationship.cascade_delete() {
                f.push_str(" ON DELETE CASCADE");
            }

            foreigns.push(f);
        }

        entry.push_str(&the_type);
        entry
    }

    fn build_query(&self, query: &QueryBuilder, params: &mut Vec<String>) -> String {
        let mut sql = "SELECT".to_owned();

        // fields
        match query.select_columns() {
            Some(fields) => sql = format!("{} {}", sql, fields.join(",")),
            None => {
                if query.all_columns() {
                    sql = format!("{} *", sql) // Select all columns by default
                }
            }
        };

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
        if let Some(joins) = query.joins() {
            for a_join in joins {
                sql = format!(
                    "{} {} join {} on {}",
                    sql,
                    a_join.join_type(),
                    a_join.table(),
                    a_join.join_clause()
                );
            }
        }

        // wheres
        sql = format!("{} {}", sql, self.build_where_clauses(query, params));

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

    fn transform_condition(&self, condition: &Condition, params: &mut Vec<String>) -> String {
        self.transform_value(condition.value(), params);

        let placeholder =
            if *condition.operator() == Operator::In || *condition.operator() == Operator::NotIn {
                let length = match &condition.value() {
                    QueryValue::Field(field) => match field {
                        FieldValue::I64s(v) => v.len(),
                        FieldValue::U64s(v) => v.len(),
                        FieldValue::Strings(v) => v.len(),
                        FieldValue::Array(v) => v.len(),
                        _ => 1,
                    },
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

    fn row_to_json(&self, row: &SqliteRow) -> serde_json::Value {
        let mut this_row = serde_json::Map::new();

        // types are from : https://docs.rs/sqlx/latest/sqlx/mysql/types/index.html
        for col in row.columns() {
            let name = col.name().to_owned();
            match col.type_info().to_string().as_str() {
                "BOOLEAN" | "TINYINT(1)" => {
                    let v: bool = row.get(col.name());
                    this_row.insert(name, serde_json::Value::Bool(v));
                }
                "TINYINT" => {
                    let v = row.try_get::<i8, &str>(col.name());
                    if let Ok(v) = v {
                        this_row
                            .insert(name, serde_json::Value::Number(serde_json::Number::from(v)));
                    } else {
                        this_row.insert(
                            name,
                            serde_json::Value::Number(serde_json::Number::from(0_i8)),
                        );
                    }
                }
                "SMALLINT" => {
                    let v = row.try_get::<i16, &str>(col.name());
                    if let Ok(v) = v {
                        this_row
                            .insert(name, serde_json::Value::Number(serde_json::Number::from(v)));
                    } else {
                        this_row.insert(
                            name,
                            serde_json::Value::Number(serde_json::Number::from(0_i16)),
                        );
                    }
                }
                "INT" => {
                    let v = row.try_get::<i32, &str>(col.name());
                    if let Ok(v) = v {
                        this_row
                            .insert(name, serde_json::Value::Number(serde_json::Number::from(v)));
                    } else {
                        this_row.insert(
                            name,
                            serde_json::Value::Number(serde_json::Number::from(0_i32)),
                        );
                    }
                }
                "BIGINT" => {
                    let v = row.try_get::<i64, &str>(col.name());
                    if let Ok(v) = v {
                        this_row
                            .insert(name, serde_json::Value::Number(serde_json::Number::from(v)));
                    } else {
                        this_row.insert(
                            name,
                            serde_json::Value::Number(serde_json::Number::from(0_i64)),
                        );
                    }
                }
                "TINYINT UNSIGNED" => {
                    let v = row.try_get::<u8, &str>(col.name());
                    if let Ok(v) = v {
                        this_row
                            .insert(name, serde_json::Value::Number(serde_json::Number::from(v)));
                    } else {
                        this_row.insert(
                            name,
                            serde_json::Value::Number(serde_json::Number::from(0_u8)),
                        );
                    }
                }
                "SMALLINT UNSIGNED" => {
                    let v = row.try_get::<u16, &str>(col.name());
                    if let Ok(v) = v {
                        this_row
                            .insert(name, serde_json::Value::Number(serde_json::Number::from(v)));
                    } else {
                        this_row.insert(
                            name,
                            serde_json::Value::Number(serde_json::Number::from(0_u16)),
                        );
                    }
                }
                "INT UNSIGNED" => {
                    let v = row.try_get::<u32, &str>(col.name());
                    if let Ok(v) = v {
                        this_row
                            .insert(name, serde_json::Value::Number(serde_json::Number::from(v)));
                    } else {
                        this_row.insert(
                            name,
                            serde_json::Value::Number(serde_json::Number::from(0_u32)),
                        );
                    }
                }
                // "BIGINT UNSIGNED" => {
                //     let v = row.try_get::<u64, &str>(col.name());
                //     if let Ok(v) = v {
                //         this_row.insert(
                //             col.name().to_owned(),
                //             serde_json::Value::Number(serde_json::Number::from(v)),
                //         );
                //     } else {
                //         this_row.insert(
                //             col.name().to_owned(),
                //             serde_json::Value::Number(serde_json::Number::from(0_u64)),
                //         );
                //     }
                // }
                "DOUBLE" | "FLOAT" => {
                    let v = row.try_get::<f64, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(
                            name,
                            serde_json::Value::Number(serde_json::Number::from_f64(v).unwrap()),
                        );
                    } else {
                        this_row.insert(
                            name,
                            serde_json::Value::Number(
                                serde_json::Number::from_f64(0.0_f64).unwrap(),
                            ),
                        );
                    }
                }
                "CHAR" | "VARCHAR" | "TEXT" => {
                    if let Ok(v) = row.try_get::<String, &str>(col.name()) {
                        this_row.insert(name, serde_json::Value::String(v));
                    } else {
                        this_row.insert(name, serde_json::Value::Null);
                    }
                }
                "TIMESTAMP" => {
                    let v = row.try_get::<chrono::DateTime<chrono::Utc>, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, serde_json::Value::String(v.to_string()));
                    } else {
                        this_row.insert(name, serde_json::Value::Null);
                    }
                }
                "DATE" => {
                    let v = row.try_get::<chrono::NaiveDate, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, serde_json::Value::String(v.to_string()));
                    } else {
                        this_row.insert(name, serde_json::Value::Null);
                    }
                }
                "TIME" => {
                    let v = row.try_get::<chrono::NaiveTime, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, serde_json::Value::String(v.to_string()));
                    } else {
                        this_row.insert(name, serde_json::Value::Null);
                    }
                }
                "DATETIME" => {
                    let v = row.try_get::<chrono::NaiveDateTime, &str>(col.name());

                    if let Ok(v) = v {
                        this_row.insert(
                            col.name().to_owned(),
                            serde_json::Value::String(v.to_string()),
                        );
                    } else {
                        this_row.insert(col.name().to_owned(), serde_json::Value::Null);
                    }
                }
                "VARBINARY" | "BINARY" | "BLOB" => {
                    // TODO find a means to represent binary
                }
                _ => {
                    log::debug!("not mapped {:#?}", col.type_info());
                }
            }
        }

        serde_json::Value::Object(this_row)
    }

    fn row_to_insert_value(&self, row: &SqliteRow) -> ColumnAndValue {
        let mut this_row = HashMap::new();

        for col in row.columns() {
            let name = col.name().to_owned();
            match col.type_info().to_string().to_ascii_uppercase().as_str() {
                "BOOLEAN" | "TINYINT(1)" => {
                    let v: bool = row.get(col.name());
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
                "BIGINT" | "INT64" | "INTEGER" => {
                    let v = row.try_get::<i64, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, v.into());
                    } else {
                        this_row.insert(name, 0_i64.into());
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
                        this_row.insert(name, v.to_string().into());
                    } else {
                        this_row.insert(name, FieldValue::Null);
                    }
                }
                "DATE" => {
                    let v = row.try_get::<chrono::NaiveDate, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(col.name().to_owned(), FieldValue::Date(v));
                    } else {
                        this_row.insert(col.name().to_owned(), FieldValue::Null);
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
                    log::debug!(
                        "not mapped {:#?}",
                        col.type_info().to_string().to_ascii_uppercase()
                    );
                }
            }
        }
        this_row
    }
}
