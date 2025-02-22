use crate::base::{
    column::{ColumnBlueprint, ColumnDefault, ColumnType},
    index::IndexType,
    query::{QueryAction, QueryBuilder},
    query_conditions::Condition,
    query_operators::Operator,
    schema::{DatabaseKind, RelationalDbTrait, SchemaManagerTrait},
    table::{TableBlueprint, UPDATED_AT_FIELD},
};
use crate::{field_values::FieldValue, query_values::QueryValue, types::ColumnAndValue};
use anyhow::anyhow;
use async_trait::async_trait;
use futures::stream::TryStreamExt;
use sqlx::{
    Arguments, Column, Pool, Row, Sqlite, TypeInfo,
    sqlite::{SqliteArguments, SqliteRow},
    types::chrono,
};
use std::{collections::HashMap, sync::Arc};

const LOG_TARGET: &str = "sqlite_db_driver";
pub const SQLITE_KIND: &str = "sqlite";

pub struct SqliteSchemaManager {
    db_pool: Arc<Pool<Sqlite>>,
}

impl SqliteSchemaManager {
    pub fn new(db_pool: Arc<Pool<Sqlite>>) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl RelationalDbTrait for SqliteSchemaManager {
    fn kind(&self) -> DatabaseKind {
        SQLITE_KIND.into()
    }
}

#[async_trait]
impl SchemaManagerTrait for SqliteSchemaManager {
    fn fetch_table_for_update(&self, name: &str) -> TableBlueprint {
        TableBlueprint::new(name)
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

    async fn stream_result(
        &self,
        query_builder: &QueryBuilder,
        sender: tokio::sync::mpsc::Sender<ColumnAndValue>,
    ) {
        let mut params = SqliteArguments::default();
        let statement = self.build_query(query_builder, &mut params);

        let query = sqlx::query_with(&statement, params);

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
            let query = QueryBuilder::new(name, QueryAction::DropTable);
            return self.execute(query).await.is_ok();
        }
        false
    }

    async fn apply(&self, table: TableBlueprint) {
        self.do_apply(table).await
    }

    async fn execute(&self, query: QueryBuilder) -> anyhow::Result<()> {
        self.do_execute(query).await
    }

    async fn fetch_all(
        &self,
        query_builder: &QueryBuilder,
    ) -> Result<Option<Vec<HashMap<String, FieldValue>>>, anyhow::Error> {
        let mut results = Vec::new();

        let mut params = SqliteArguments::default();
        let statement = self.build_query(query_builder, &mut params);

        let query = sqlx::query_with(&statement, params);

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
        let mut params = SqliteArguments::default();

        let statement = self.build_query(query_builder, &mut params);
        let query = sqlx::query_with(&statement, params);

        return match query.fetch_optional(self.db_pool.as_ref()).await {
            Ok(result) => match result {
                Some(row) => Ok(Some(self.row_to_column_value(&row))),
                None => Ok(None),
            },
            Err(e) => Err(e.into()),
        };
    }

    async fn raw_insert(&self, sql: &str, row: Vec<FieldValue>) -> Result<bool, anyhow::Error> {
        let mut query = sqlx::query(sql);
        for field in row {
            query = query.bind(field.to_string());
        }
        match query.execute(self.db_pool.as_ref()).await {
            Ok(_) => Ok(true),
            Err(e) => Err(e.into()),
        }
    }

    async fn raw_update(&self, sql: &str, params: Vec<FieldValue>) -> Result<u64, anyhow::Error> {
        let mut query = sqlx::query(sql);
        for p in params {
            query = query.bind(p.to_string());
        }

        match query.execute(self.db_pool.as_ref()).await {
            Ok(v) => Ok(v.rows_affected()),
            Err(e) => Err(e.into()),
        }
    }

    async fn raw_delete(&self, sql: &str, params: Vec<FieldValue>) -> Result<u64, anyhow::Error> {
        self.raw_update(sql, params).await
    }

    async fn raw_select(
        &self,
        sql: &str,
        params: Vec<FieldValue>,
    ) -> Result<Vec<ColumnAndValue>, anyhow::Error> {
        let mut results = Vec::new();
        let mut query = sqlx::query(sql);

        for p in &params {
            query = query.bind(p.to_string());
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

        Ok(results)
    }

    async fn raw_statement(&self, sql: &str) -> Result<bool, anyhow::Error> {
        let query = sqlx::query(sql);

        match query.execute(self.db_pool.as_ref()).await {
            Ok(_v) => Ok(true),
            Err(e) => Err(e.into()),
        }
    }
}

impl SqliteSchemaManager {
    async fn do_apply(&self, table: TableBlueprint) {
        if table.view_query.is_some() {
            // working with view table
            self.create_or_replace_view(table).await
        } else {
            // working with real table
            self.apply_table_changes(table).await
        }
    }

    async fn do_execute(&self, query: QueryBuilder) -> anyhow::Result<()> {
        let mut params = SqliteArguments::default();

        let mut sql;
        match query.action() {
            QueryAction::Create {
                rows,
                do_soft_insert,
            } => {
                sql = format!(
                    "INSERT {} INTO {} ",
                    if *do_soft_insert { "OR IGNORE" } else { "" },
                    query.table()
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
                        keys.iter().for_each(|col| {
                            let field = a_row.1.get(col).unwrap();
                            self.field_value_to_args(field, &mut params);
                        });
                        let separator = if a_row.0 > 0 { "," } else { "" };

                        sql = format!("{} {} ({})", sql, separator, &placeholders);
                    }
                }
            }
            QueryAction::Update(column_values) => {
                let mut columns = Vec::new();
                for entry in column_values {
                    if *entry.1 != FieldValue::NotSet {
                        columns.push(entry.0);
                        self.field_value_to_args(entry.1, &mut params);
                    }
                }
                sql = format!("UPDATE `{}` SET ", query.table());
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
                sql = format!("DELETE {0} FROM {0} ", query.table());
                // joins
                sql = format!("{} {}", sql, self.build_join(&query, &mut params));
                // where
                sql = format!("{} {}", sql, self.build_where_clauses(&query, &mut params));
            }
            QueryAction::DropTable => {
                sql = format!("DROP TABLE IF EXISTS {};", query.table());
            }
            QueryAction::RenameColumn { old, new } => {
                let table = query.table();
                sql = format!("ALTER TABLE {} RENAME COLUMN {} TO {}", table, old, new);
            }
            QueryAction::RenameTable(new) => {
                let table = query.table();
                sql = format!("ALTER TABLE {} RENAME TO {}", table, new);
            }
            QueryAction::DropColumn(column) => {
                let table = query.table();
                sql = format!("ALTER TABLE {} DROP {}", table, column);
            }
            _ => {
                sql = "".into();
            }
        }

        let result = sqlx::query_with(&sql, params)
            .execute(self.db_pool.as_ref())
            .await;

        match result {
            Ok(r) => {
                log::debug!("{} result: {:#?}", query.action(), r);
                Ok(())
            }
            Err(e) => {
                log::error!("{} failed: {}", query.action(), e);
                Err(anyhow!(e))
            }
        }
    }

    async fn create_or_replace_view(&self, table: TableBlueprint) {
        if let Some(query) = &table.view_query {
            let mut params = SqliteArguments::default();
            let sql = self.build_query(query, &mut params);

            let query = format!("CREATE OR REPLACE VIEW `{}` AS ({})", &table.name, sql);

            let result = sqlx::query_with(&query, params)
                .execute(self.db_pool.as_ref())
                .await;
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

    async fn apply_table_changes(&self, table: TableBlueprint) {
        let mut foreign = Vec::new();
        let columns: Vec<String> = table
            .columns()
            .iter()
            .map(|column| self.create_column(column, &mut foreign))
            .collect();

        let mut query = if table.is_new() {
            format!("CREATE TABLE `{}`", &table.name)
        } else {
            format!("ALTER TABLE `{}`", &table.name)
        };

        if !columns.is_empty() {
            query = if foreign.is_empty() {
                format!("{} ({})", query, columns.join(","))
            } else {
                format!("{} ({}, {})", query, columns.join(","), foreign.join(","))
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

    fn create_column(&self, column: &ColumnBlueprint, foreign: &mut Vec<String>) -> String {
        let mut entry = format!("`{}`", &column.name);
        let mut the_type = " ".to_owned();

        // column type
        match column.column_type {
            ColumnType::AutoIncrementId => {
                the_type.push_str("INTEGER");
                foreign.push(format!("PRIMARY KEY('{}' AUTOINCREMENT)", &column.name));
            }
            ColumnType::Boolean => the_type.push_str("BOOLEAN"),
            ColumnType::Char(length) => the_type.push_str(&format!("VARCHAR({})", length)),
            ColumnType::Datetime => the_type.push_str("datetime"),
            ColumnType::Timestamp => the_type.push_str("timestamp"),
            ColumnType::Float => the_type.push_str("double"),
            ColumnType::Integer => the_type.push_str("INTEGER"),
            ColumnType::Json => the_type.push_str("json"),
            ColumnType::Number => the_type.push_str("double"),
            ColumnType::Binary => the_type.push_str("BLOB"),
            ColumnType::String(length) => {
                let q = format!("VARCHAR({})", length);
                the_type.push_str(q.as_str());
            }
            ColumnType::Text => the_type.push_str("TEXT"),
            ColumnType::Uuid => the_type.push_str("uuid"),
            ColumnType::Enum(ref opt) => {
                if column.check.is_none() {
                    let list = opt
                        .iter()
                        .map(|e| format!("'{}'", e))
                        .collect::<Vec<String>>()
                        .join(",");
                    the_type.push_str(&format!(
                        "varchar(255) CONSTRAINT {0}_chk check (\"{0}\" in ({1}))",
                        column.name, list
                    ));
                } else {
                    the_type.push_str("varchar(255)"); // the check will be added below
                }
            }
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
                ColumnDefault::EmptyArray => the_type.push_str("'[]'"),
                ColumnDefault::EmptyObject => the_type.push_str("'{}'"),
                ColumnDefault::EmptyString => the_type.push_str("''"),
                ColumnDefault::Uuid => the_type.push_str("GUID()"),
                ColumnDefault::Ulid => (),
                ColumnDefault::UpdatedAt => the_type.push_str("CURRENT_TIMESTAMP"),
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

            foreign.push(f);
        }

        // column constrain check
        if let Some(check) = &column.check {
            match column.column_type {
                ColumnType::Enum(ref opt) => {
                    let list = opt
                        .iter()
                        .map(|e| format!("'{}'", e))
                        .collect::<Vec<String>>()
                        .join(",");
                    the_type.push_str(&format!(
                        " CONSTRAINT {0}_chk CHECK ({1} AND \"{0}\" in ({2}) )",
                        &column.name, check, list
                    ));
                }
                _ => {
                    the_type.push_str(&format!(
                        " CONSTRAINT {}_chk CHECK ({})",
                        &column.name, check
                    ));
                }
            }
        }

        entry.push_str(&the_type);
        entry
    }

    fn build_query(&self, query: &QueryBuilder, params: &mut SqliteArguments) -> String {
        let mut sql = "SELECT".to_owned();

        // fields
        if let QueryAction::Query { columns } = query.action() {
            if let Some(fields) = columns {
                sql = format!("{} {}", sql, fields.join(","));
            } else {
                sql = format!("{} *", sql) // Select all columns by default
            }
        }

        // join fields
        if let Some(joins) = query.joins() {
            for a_join in joins {
                if let Some(columns) = a_join.select_columns() {
                    sql = format!("{}, {}", sql, columns.join(","));
                }
            }
        }

        // from
        sql = format!("{} FROM {}", sql, query.table());

        // joins
        sql = format!("{} {}", sql, self.build_join(query, params));

        // wheres
        sql = format!("{} {}", sql, self.build_where_clauses(query, params));

        // group by

        // order by
        if let Some(order) = self.build_order_by(query) {
            sql = format!("{} {}", sql, order);
        }

        // limit
        if let Some(limit) = query.limit_by() {
            sql = format!("{} {}", sql, limit);
        }

        // TODO: offset

        sql
    }

    fn build_join(&self, query: &QueryBuilder, _params: &mut SqliteArguments) -> String {
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

    fn build_order_by(&self, query: &QueryBuilder) -> Option<String> {
        query.order_by().map(|order| order.to_string())
    }

    fn build_where_clauses(&self, query: &QueryBuilder, params: &mut SqliteArguments) -> String {
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

    fn transform_condition(&self, condition: &Condition, params: &mut SqliteArguments) -> String {
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

    fn transform_value(&self, value: &QueryValue, params: &mut SqliteArguments) {
        match value {
            QueryValue::SubQuery(q) => {
                self.build_query(q, params);
            }
            QueryValue::Field(field) => self.field_value_to_args(field, params),
        }
    }

    fn row_to_column_value(&self, row: &SqliteRow) -> ColumnAndValue {
        let mut this_row = HashMap::new();

        for col in row.columns() {
            let name = col.name().to_owned();
            match col.type_info().name() {
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
                "BIGINT" | "INT64" | "INTEGER" => {
                    let v = row.try_get::<i64, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, v.into());
                    } else {
                        this_row.insert(name, 0_i64.into());
                    }
                }
                "REAL" | "DOUBLE" | "FLOAT" => {
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
                "VARBINARY" | "BINARY" | "BLOB" | "BYTEA" => {
                    let v = row.try_get::<String, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(
                            col.name().to_string(),
                            FieldValue::Binary(hex::decode(v).unwrap()),
                        );
                    } else {
                        this_row.insert(col.name().to_string(), FieldValue::Binary(vec![]));
                    }
                }
                "NULL" => {
                    if let Ok(v) = row.try_get::<i64, &str>(col.name()) {
                        this_row.insert(name, v.into());
                    } else if let Ok(v) = row.try_get::<f64, &str>(col.name()) {
                        this_row.insert(name, v.into());
                    } else if let Ok(v) = row.try_get::<String, &str>(col.name()) {
                        this_row.insert(name, v.into());
                    }
                }
                _ => {
                    tracing::debug!(
                        "unsupported field type : {:?} => value: {:#?}",
                        name,
                        col.type_info()
                    );
                }
            }
        }
        this_row
    }

    fn field_value_to_args(&self, field: &FieldValue, params: &mut SqliteArguments) {
        match field.clone() {
            // sqlite arguments uses a lifetime
            FieldValue::DateTime(dt) => {
                _ = Arguments::add(params, dt); // format!("{}", dt.format("%F %T")));
            }
            FieldValue::Timestamp(dt) => {
                _ = Arguments::add(params, dt); //format!("{}", dt.format("%F %T")));
            }
            FieldValue::Date(d) => {
                _ = Arguments::add(params, d); //format!("{}", d.format("%F")));
            }
            FieldValue::Binary(d) => {
                _ = Arguments::add(params, d);
            }
            FieldValue::Object(d) => {
                _ = Arguments::add(params, sqlx::types::Json(d));
            }
            FieldValue::F64(v) => {
                _ = Arguments::add(params, v);
            }
            FieldValue::I64(v) => {
                _ = Arguments::add(params, v);
            }
            FieldValue::String(v) => {
                _ = Arguments::add(params, sqlx::types::Text(v));
            }
            FieldValue::Array(v) => {
                for entry in v {
                    self.field_value_to_args(&entry, params);
                }
            }
            FieldValue::Boolean(v) => {
                _ = Arguments::add(params, v);
            }
            FieldValue::Time(t) => {
                _ = Arguments::add(params, t);
            }
            FieldValue::U64(v) => {
                let v = v as i64;
                _ = Arguments::add(params, v);
            }
            FieldValue::Null => {
                _ = Arguments::add(params, "NULL");
            }
            FieldValue::NotSet => (),
        }
    }
}
