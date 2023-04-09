use crate::base::{
    column::{BaseColumn, ColumnDefault, ColumnType},
    query::QueryBuilder,
    query_conditions::Condition,
    query_operators::Operator,
    query_values::Value,
    schema::{RelationalDbTrait, SchemaManagerTrait},
    table::BaseTable,
};
use async_trait::async_trait;
use futures::stream::TryStreamExt;
use sqlx::{any::AnyKind, mysql::MySqlRow, types::chrono, Column, MySql, Pool, Row};
use std::sync::Arc;

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
pub struct MySqlSchemaManager {
    db_pool: Arc<Pool<MySql>>,
    active_query: Option<ActiveQuery>,
}

impl MySqlSchemaManager {
    pub fn new(db_pool: Arc<Pool<MySql>>) -> Self {
        Self {
            db_pool,
            active_query: None,
        }
    }
}

#[async_trait]
impl RelationalDbTrait for MySqlSchemaManager {
    fn instance(db_pool: Arc<Pool<MySql>>) -> Self
    where
        Self: Sized,
    {
        Self::new(db_pool)
    }

    fn kind(&self) -> AnyKind {
        AnyKind::MySql
        // self.db_pool.any_kind()
    }
}

#[async_trait]
impl SchemaManagerTrait for MySqlSchemaManager {
    fn fetch_table_for_update(&self, name: &str) -> BaseTable {
        BaseTable::new(name)
    }
    async fn has_table(&self, name: &str) -> bool {
        let query = "SELECT table_name FROM INFORMATION_SCHEMA.TABLES WHERE table_name = ?";

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

    async fn save(&self, query: QueryBuilder) {
        self.do_save(query).await
    }

    async fn fetch_all_as_json(&self) -> Vec<serde_json::Value> {
        let mut results = Vec::new();
        match &self.active_query {
            Some(active_query) => {
                let mut query = sqlx::query(&active_query.statement);
                for p in &active_query.params {
                    query = query.bind::<&str>(p);
                }

                let mut rows = query.fetch(self.db_pool.as_ref());
                while let Some(row) = rows.try_next().await.ok().unwrap_or_default() {
                    results.push(self.row_to_json(&row));
                }
            }
            None => (),
        }

        results
    }
}

impl MySqlSchemaManager {
    async fn do_commit(&self, table: BaseTable) {
        if table.view_query.is_some() {
            // working with view table
            self.create_or_replace_view(table).await
        } else {
            // working with real table
            self.apply_table_changes(table).await
        }
    }

    async fn do_save(&self, query: QueryBuilder) {
        let mut columns = Vec::new();
        let mut params = Vec::new();
        if let Some(list) = query.set_columns() {
            for entry in list {
                columns.push(entry.0);
                params.push(entry.1.to_string());
            }
        }

        let mut sql;
        if !query.where_clauses().is_empty() {
            sql = format!("UPDATE `{}` SET ", query.tables().join(","));
            for entry in columns.iter().enumerate() {
                if entry.0 > 0 {
                    sql = format!("{}, `{}` = ? ", sql, *entry.1);
                } else {
                    sql = format!("{} `{}` = ? ", sql, *entry.1);
                }
            }

            sql = format!("{} {}", sql, self.build_where_clauses(&query, &mut params));
        } else {
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

        let mut query_statement = sqlx::query(&sql);

        for p in &params {
            query_statement = query_statement.bind(p);
        }

        let result = query_statement.execute(self.db_pool.as_ref()).await;

        match result {
            Ok(r) => {
                dbg!(r);
            }
            Err(e) => {
                dbg!(e);
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
                the_type.push_str("bigint(20) unsigned AUTO_INCREMENT PRIMARY KEY")
            }
            ColumnType::Boolean => the_type.push_str("tinyint(1)"),
            ColumnType::Char(length) => {
                the_type.push_str(&format!("char({}) COLLATE 'utf8mb4_unicode_ci'", length))
            }
            ColumnType::Date => the_type.push_str("datetime"),
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
        match query.select_columns() {
            Some(fields) => sql = format!("{} {}", sql, fields.join(",")),
            None => (),
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
                    Value::I64s(v) => v.len(),
                    Value::U64s(v) => v.len(),
                    Value::Strings(v) => v.len(),
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

    fn transform_value(&self, value: &Value, params: &mut Vec<String>) {
        match value {
            Value::SubQuery(q) => {
                self.build_query(q, params);
            }
            _ => value.to_param(params),
        }
    }

    fn row_to_json(&self, row: &MySqlRow) -> serde_json::Value {
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
                "BIGINT UNSIGNED" => {
                    let v = row.try_get::<u64, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(
                            col.name().to_owned(),
                            serde_json::Value::Number(serde_json::Number::from(v)),
                        );
                    } else {
                        this_row.insert(
                            col.name().to_owned(),
                            serde_json::Value::Number(serde_json::Number::from(0_u64)),
                        );
                    }
                }
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
                    // TODO find a mean to represent binary
                }
                _ => {
                    dbg!("not mapped {:#?}", col.type_info());
                }
            }
        }

        serde_json::Value::Object(this_row)
    }
}
