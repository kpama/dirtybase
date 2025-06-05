use crate::base::{
    column::{ColumnBlueprint, ColumnDefault, ColumnType},
    query::{QueryAction, QueryBuilder},
    query_conditions::Condition,
    query_operators::Operator,
    schema::{DatabaseKind, RelationalDbTrait, SchemaManagerTrait},
    table::TableBlueprint,
};
use crate::{field_values::FieldValue, query_values::QueryValue, types::ColumnAndValue};
use anyhow::anyhow;
use async_trait::async_trait;
use dirtybase_contract::db_contract::{
    base::{aggregate::Aggregate, index::IndexType},
    query_column::{QueryColumn, QueryColumnName},
};
use futures::stream::TryStreamExt;
use sqlx::{
    Arguments, Column, Pool, Postgres, Row,
    postgres::{PgArguments, PgRow},
    types::chrono,
};
use std::{collections::HashMap, sync::Arc};

const LOG_TARGET: &str = "postgresql_db_driver";
pub const POSTGRES_KIND: &str = "postgres";

pub struct PostgresSchemaManager {
    db_pool: Arc<Pool<Postgres>>,
}

impl PostgresSchemaManager {
    pub fn new(db_pool: Arc<Pool<Postgres>>) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl RelationalDbTrait for PostgresSchemaManager {
    fn kind(&self) -> DatabaseKind {
        POSTGRES_KIND.into()
    }
}

#[async_trait]
impl SchemaManagerTrait for PostgresSchemaManager {
    fn fetch_table_for_update(&self, name: &str) -> TableBlueprint {
        TableBlueprint::new(name)
    }
    async fn has_table(&self, name: &str) -> Result<bool, anyhow::Error> {
        let query = "SELECT EXISTS ( SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = $1);";

        let result = sqlx::query(query)
            .bind(name)
            .map(|row: PgRow| {
                if let Ok(exists) = row.try_get::<bool, &str>("exists") {
                    return exists;
                }
                false
            })
            .fetch_one(self.db_pool.as_ref())
            .await;

        result.map_err(|e| anyhow::anyhow!(e))
    }

    async fn stream_result(
        &self,
        query_builder: &QueryBuilder,
        sender: tokio::sync::mpsc::Sender<ColumnAndValue>,
    ) -> Result<(), anyhow::Error> {
        let mut params = PgArguments::default();

        let statement = self.build_query(query_builder, &mut params)?;

        let query = sqlx::query_with(&statement, params);
        // for p in &params {
        //     query = query.bind::<&str>(p);
        // }

        let mut rows = query.fetch(self.db_pool.as_ref());
        while let Ok(result) = rows.try_next().await {
            if let Some(row) = result {
                if let Err(e) = sender.send(self.row_to_column_value(&row)).await {
                    log::error!(target: LOG_TARGET, "could not send mpsc stream: {}", &e);
                    return Err(anyhow::anyhow!(e));
                }
            } else {
                break;
            }
        }

        Ok(())
    }

    async fn drop_table(&self, name: &str) -> Result<(), anyhow::Error> {
        if self.has_table(name).await? {
            let query = QueryBuilder::new(name, QueryAction::DropTable);
            return self.execute(query).await;
        }

        Err(anyhow!("could not drop the table"))
    }

    async fn apply(&self, table: TableBlueprint) -> Result<(), anyhow::Error> {
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
        let mut params = PgArguments::default();

        let statement = self.build_query(query_builder, &mut params)?;

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
        let mut params = PgArguments::default();
        let statement = self.build_query(query_builder, &mut params)?;

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
            Err(e) => Err(e.into()),
            _ => Ok(true),
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
        _sql: &str,
        _params: Vec<FieldValue>,
    ) -> Result<Vec<ColumnAndValue>, anyhow::Error> {
        todo!();
    }

    async fn raw_statement(&self, _sql: &str) -> Result<bool, anyhow::Error> {
        todo!();
    }
}

impl PostgresSchemaManager {
    async fn do_apply(&self, table: TableBlueprint) -> Result<(), anyhow::Error> {
        return if table.view_query.is_some() {
            // working with view table
            self.create_or_replace_view(table).await
        } else {
            // working with real table
            self.apply_table_changes(table).await
        };
    }

    async fn do_execute(&self, query: QueryBuilder) -> anyhow::Result<()> {
        let mut params = PgArguments::default();

        let mut sql;
        match query.action() {
            QueryAction::Create {
                rows,
                do_soft_insert,
            } => {
                sql = format!(
                    "INSERT {} INTO \"{}\" ",
                    if *do_soft_insert { "IGNORE" } else { "" },
                    query.table()
                );
                sql = self.build_insert_data(&mut params, rows, sql)?;
            }
            QueryAction::Upsert {
                rows,
                unique,
                to_update,
            } => {
                sql = format!("INSERT INTO {}", query.table());
                sql = self.build_insert_data(&mut params, rows, sql)?;

                if !unique.is_empty() && !to_update.is_empty() {
                    sql = format!(
                        "{} ON CONFLICT ({}) ",
                        sql,
                        unique
                            .iter()
                            .map(|e| format!("\"{}\"", e))
                            .collect::<Vec<String>>()
                            .join(",")
                    );

                    let mut update_values = Vec::new();
                    for entry in to_update {
                        update_values.push(format!("\"{0}\" = \"excluded\".\"{0}\"", entry));
                    }

                    sql = format!("{} DO UPDATE SET {}", sql, update_values.join(","));
                }
            }

            QueryAction::Update(column_values) => {
                let mut columns = Vec::new();
                for entry in column_values {
                    if *entry.1 != FieldValue::NotSet {
                        columns.push(entry.0);
                        self.field_value_to_args(entry.1, &mut params)?;
                    }
                }
                sql = format!("UPDATE \"{}\" SET ", query.table());
                for entry in columns.iter().enumerate() {
                    let index = entry.0 + 1;
                    if entry.0 > 0 {
                        sql = format!("{}, \"{}\" = ${} ", sql, *entry.1, index);
                    } else {
                        sql = format!("{} \"{}\" = ${} ", sql, *entry.1, index);
                    }
                }

                // joins
                sql = format!("{} {}", sql, self.build_join(&query)?);
                // where
                sql = format!("{} {}", sql, self.build_where_clauses(&query, &mut params)?);
            }
            QueryAction::Delete => {
                sql = format!("DELETE FROM {0} ", query.table());
                // joins
                sql = format!("{} {}", sql, self.build_join(&query)?);
                // where
                sql = format!("{} {}", sql, self.build_where_clauses(&query, &mut params)?);
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

    async fn create_or_replace_view(&self, table: TableBlueprint) -> Result<(), anyhow::Error> {
        if let Some(query) = &table.view_query {
            let mut params = PgArguments::default();
            let sql = self.build_query(query, &mut params)?;

            let query = format!("CREATE OR REPLACE VIEW \"{}\" AS ({})", &table.name, sql);

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
                    return Err(anyhow::anyhow!(error));
                }
            }
        }

        Ok(())
    }

    async fn apply_table_changes(&self, table: TableBlueprint) -> Result<(), anyhow::Error> {
        let mut query = String::new();
        let columns: Vec<String> = table
            .columns()
            .iter()
            .map(|column| self.create_column(column))
            .collect();

        query = if table.is_new() {
            format!("{} CREATE TABLE IF NOT EXISTS \"{}\"", &query, &table.name)
        } else {
            format!("{} ALTER TABLE \"{}\"", &query, &table.name)
        };

        if table.is_new() {
            if !columns.is_empty() {
                query = format!("{} ({})", query, columns.join(","));
            }
        } else {
            if !columns.is_empty() {
                query = format!("{} ADD {}", query, columns.join(","));
            }
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
                log::error!("Could not {} table {}: {}", action, name, &e);
                return Err(anyhow::anyhow!(
                    "Could not {} table {}: {}",
                    action,
                    name,
                    &e
                ));
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
                                index
                                    .columns()
                                    .iter()
                                    .map(|col| { format!("\"{}\"", col) })
                                    .collect::<Vec<String>>()
                                    .join(",")
                            );
                        }
                    }
                    IndexType::Unique(index) => {
                        if index.delete_index() {
                            sql = format!("DROP INDEX IF EXISTS {}.{}", &table.name, index.name());
                        } else {
                            sql = format!(
                                "CREATE UNIQUE  INDEX IF NOT EXISTS \"{}\" ON \"{}\" ({})",
                                index.name(),
                                &table.name,
                                index
                                    .columns()
                                    .iter()
                                    .map(|col| { format!("\"{}\"", col) })
                                    .collect::<Vec<String>>()
                                    .join(",")
                            );
                        }
                    }
                }

                let index_result = sqlx::query(&sql).execute(self.db_pool.as_ref()).await;
                match index_result {
                    Ok(_) => log::info!("table index created"),
                    Err(e) => {
                        log::error!("postgres: {}", &sql);
                        log::error!("could not create table index: {}", &e);
                        return Err(anyhow::anyhow!("could not create table index: {}", &e));
                    }
                }
            }
        }

        Ok(())
    }

    fn create_column(&self, column: &ColumnBlueprint) -> String {
        let mut entry = format!("\"{}\"", &column.name);
        let mut the_type = " ".to_owned();

        // column type
        match column.column_type {
            ColumnType::AutoIncrementId => the_type.push_str("BIGSERIAL PRIMARY KEY"),
            ColumnType::Boolean => the_type.push_str("BOOLEAN"),
            ColumnType::Char(_) => the_type.push_str("VARCHAR"),
            ColumnType::Timestamp | ColumnType::Datetime => the_type.push_str("TIMESTAMPTZ"),
            ColumnType::Date => the_type.push_str("DATE"),
            ColumnType::Integer => the_type.push_str("BIGINT"),
            ColumnType::Json => the_type.push_str("JSONB"),
            ColumnType::Number | ColumnType::Float => the_type.push_str("DOUBLE PRECISION"),
            ColumnType::Binary => the_type.push_str("BYTEA"),
            ColumnType::String(length) => {
                let q = format!("VARCHAR({})", length);
                the_type.push_str(q.as_str());
            }
            ColumnType::Text => the_type.push_str("TEXT"),
            ColumnType::Uuid => the_type.push_str("UUID"),
            ColumnType::Enum(ref opt) => {
                if column.check.is_none() {
                    let list = opt
                        .iter()
                        .map(|e| format!("'{}'", e))
                        .collect::<Vec<String>>()
                        .join(",");
                    the_type.push_str(&format!(
                        "VARCHAR(255) CONSTRAINT {0}_chk check (\"{0}\" in ({1}))",
                        column.name, list
                    ));
                } else {
                    the_type.push_str("VARCHAR(255)"); // the check will be added below
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
                ColumnDefault::Uuid => the_type.push_str("SYS_GUID()"),
                ColumnDefault::Ulid => (),
                ColumnDefault::UpdatedAt => the_type.push_str("CURRENT_TIMESTAMP"),
                ColumnDefault::Zero => the_type.push('0'),
            };
        }

        // column relationship
        if let Some(relationship) = &column.relationship {
            the_type.push_str(&format!(
                ", FOREIGN KEY (\"{}\") REFERENCES \"{}\" (\"{}\")",
                &column.name,
                &relationship.table(),
                &relationship.column()
            ));
            if relationship.cascade_delete() {
                the_type.push_str(" ON DELETE CASCADE");
            }
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

    fn build_query(
        &self,
        query: &QueryBuilder,
        params: &mut PgArguments,
    ) -> Result<String, anyhow::Error> {
        let mut sql = "SELECT".to_owned();

        // fields
        if let QueryAction::Query { columns } = query.action() {
            if let Some(fields) = columns {
                let mut col_names = Vec::new();
                for a_field in fields {
                    col_names.push(self.column_to_string(a_field, params)?);
                }
                sql = format!("{} {}", sql, col_names.join(","));
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
        sql = format!("{} {}", sql, self.build_join(query)?);

        // wheres
        sql = format!("{} {}", sql, self.build_where_clauses(query, params)?);

        // group by

        // order by
        if let Some(order) = self.build_order_by(query) {
            sql = format!("{} {}", sql, order);
        }

        // having

        // limit
        if let Some(limit) = query.limit_by() {
            sql = format!("{} {}", sql, limit);
        }

        // offset

        Ok(sql)
    }

    fn build_join(&self, query: &QueryBuilder) -> Result<String, anyhow::Error> {
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

        Ok(sql)
    }

    fn build_order_by(&self, query: &QueryBuilder) -> Option<String> {
        query.order_by().map(|order| order.to_string())
    }

    fn build_where_clauses(
        &self,
        query: &QueryBuilder,
        params: &mut PgArguments,
    ) -> Result<String, anyhow::Error> {
        let mut wheres = "".to_owned();
        for where_join in query.where_clauses() {
            wheres = where_join.as_clause(
                &wheres,
                &self.transform_condition(where_join.condition(), params)?,
            );
        }

        if !wheres.is_empty() {
            wheres = format!("WHERE {}", wheres);
        }

        Ok(wheres)
    }

    fn transform_condition(
        &self,
        condition: &Condition,
        params: &mut PgArguments,
    ) -> Result<String, anyhow::Error> {
        let placeholder;
        match condition.value() {
            QueryValue::SubQuery(sub) => {
                placeholder = self.build_query(sub, params)?;
            }
            QueryValue::ColumnName(name) => {
                placeholder = name.clone();
            }
            _ => {
                self.transform_value(condition.value(), params)?;
                placeholder = if *condition.operator() == Operator::In
                    || *condition.operator() == Operator::NotIn
                {
                    let length = match &condition.value() {
                        QueryValue::Field(FieldValue::Array(v)) => v.len(),
                        _ => 1,
                    };

                    let mut placeholder = Vec::new();
                    placeholder.resize(length, format!("${}", params.len()));
                    placeholder.join(",")
                } else {
                    format!("${}", params.len())
                };
            }
        }

        Ok(condition
            .operator()
            .as_clause(condition.column(), &placeholder))
    }

    fn transform_value(
        &self,
        value: &QueryValue,
        params: &mut PgArguments,
    ) -> Result<(), anyhow::Error> {
        match value {
            QueryValue::SubQuery(q) => {
                self.build_query(q, params)?;
            }
            QueryValue::Field(field) => self.field_value_to_args(field, params)?,
            QueryValue::ColumnName(_) => (), // does not require an entry into the params,
        }

        Ok(())
    }

    fn row_to_column_value(&self, row: &PgRow) -> ColumnAndValue {
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
                "BIGINT" | "INT8" => {
                    let v = row.try_get::<i64, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, v.into());
                    } else {
                        this_row.insert(name, 0_i64.into());
                    }
                }
                "FLOAT8" => {
                    let v = row.try_get::<f64, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, v.into());
                    } else {
                        this_row.insert(name, 0_f64.into());
                    }
                }
                "TINYINT UNSIGNED" => {
                    let v = row.try_get::<i8, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, (v as u32).into());
                    } else {
                        this_row.insert(name, 0_u32.into());
                    }
                }
                "SMALLINT UNSIGNED" => {
                    let v = row.try_get::<i16, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, (v as u32).into());
                    } else {
                        this_row.insert(name, 0_u32.into());
                    }
                }
                "INT UNSIGNED" => {
                    let v = row.try_get::<i32, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, v.into());
                    } else {
                        this_row.insert(name, 0_u32.into());
                    }
                }
                "BIGINT UNSIGNED" => {
                    let v = row.try_get::<i64, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(name, v.into());
                    } else {
                        this_row.insert(name, 0_u64.into());
                    }
                }
                "DOUBLE" | "DOUBLE PRECISION" | "FLOAT" => {
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
                "TIMESTAMP" | "TIMESTAMPTZ" => {
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
                "VARBINARY" | "BINARY" | "BLOB" | "BYTEA" => {
                    let v = row.try_get::<Vec<u8>, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(col.name().to_string(), FieldValue::Binary(v));
                    } else {
                        this_row.insert(col.name().to_string(), FieldValue::Binary(vec![]));
                    }
                }
                "JSONB" | "JSON" => {
                    if let Ok(v) = row.try_get::<serde_json::Value, &str>(col.name()) {
                        this_row.insert(col.name().to_string(), v.into());
                    } else {
                        this_row.insert(
                            col.name().to_string(),
                            serde_json::value::Value::Null.into(),
                        );
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
                "UUID" => {
                    if let Ok(v) = row.try_get::<sqlx::types::Uuid, &str>(col.name()) {
                        this_row.insert(name, v.into());
                    } else {
                        this_row.insert(name, sqlx::types::Uuid::nil().into());
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

    fn field_value_to_args(
        &self,
        field: &FieldValue,
        params: &mut PgArguments,
    ) -> Result<(), anyhow::Error> {
        build_field_value_to_args(field, params)
    }

    fn build_insert_data(
        &self,
        params: &mut PgArguments,
        rows: &[ColumnAndValue],
        mut sql: String,
    ) -> Result<String, anyhow::Error> {
        if !rows.is_empty() {
            let first_row = rows.first().unwrap();
            let keys = first_row.keys().cloned().collect::<Vec<String>>();
            let columns = keys
                .iter()
                .map(|e| format!("\"{}\"", e))
                .collect::<Vec<String>>()
                .join(",");

            sql = format!("{} ({}) VALUES ", sql, columns);
            let mut placeholder_counter = 0;
            for a_row in rows.iter().enumerate() {
                for col in &keys {
                    let field = a_row.1.get(col).unwrap();
                    self.field_value_to_args(field, params)?;
                }
                let separator = if a_row.0 > 0 { "," } else { "" };

                let placeholders = keys
                    .iter()
                    .enumerate()
                    .map(|_| {
                        placeholder_counter += 1;
                        format!("${}", placeholder_counter)
                    })
                    .collect::<Vec<String>>()
                    .join(",");
                sql = format!("{} {} ({})", sql, separator, &placeholders);
            }
        }

        Ok(sql)
    }

    fn column_to_string(
        &self,
        column: &QueryColumn,
        params: &mut PgArguments,
    ) -> Result<String, anyhow::Error> {
        let alias = column.alias().as_ref().cloned().unwrap_or_default().clone();

        if let Some(a) = column.aggregate() {
            let aggregate = match a {
                Aggregate::Avg => "AVG",
                Aggregate::Count => "COUNT",
                Aggregate::Max => "MAX",
                Aggregate::Min => "MIN",
                Aggregate::Sum => "SUM",
            };

            return match column.name() {
                QueryColumnName::Name(n) => {
                    let full_name = if let Some(tbl) = column.table() {
                        format!("\"{}\".\"{}\"", tbl, n)
                    } else {
                        n.clone()
                    };
                    if alias.is_empty() {
                        Ok(format!("({}({1})) as \"{1}\"", aggregate, full_name))
                    } else {
                        Ok(format!("{}({}) as \"{}\"", aggregate, full_name, alias))
                    }
                }
                QueryColumnName::SubQuery(query) => {
                    let sql = self.build_query(query, params)?;
                    if alias.is_empty() {
                        Ok(sql)
                    } else {
                        Ok(format!("({}({})) as \"{}\"", aggregate, sql, alias))
                    }
                }
            };
        }
        return match column.name() {
            QueryColumnName::Name(n) => {
                let full_name = if let Some(tbl) = column.table() {
                    format!("\"{}\".\"{}\"", tbl, n)
                } else {
                    n.clone()
                };

                if alias.is_empty() {
                    Ok(format!("{}", full_name))
                } else {
                    Ok(format!("{} as \"{}\"", full_name, alias))
                }
            }
            QueryColumnName::SubQuery(query) => {
                let sql = self.build_query(query, params)?;
                if alias.is_empty() {
                    Ok(sql)
                } else {
                    Ok(format!("({}) as \"{}\"", sql, alias))
                }
            }
        };
    }
}

fn build_field_value_to_args(
    field: &FieldValue,
    params: &mut PgArguments,
) -> Result<(), anyhow::Error> {
    match field {
        FieldValue::DateTime(dt) => {
            _ = Arguments::add(params, dt);
        }
        FieldValue::Timestamp(dt) => {
            _ = Arguments::add(params, dt);
        }
        FieldValue::Date(d) => {
            _ = Arguments::add(params, d);
        }
        FieldValue::Binary(d) => {
            _ = Arguments::add(params, d);
        }
        FieldValue::Uuid(d) => {
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
                build_field_value_to_args(entry, params)?;
            }
        }
        FieldValue::Boolean(v) => {
            _ = Arguments::add(params, v);
        }
        FieldValue::Time(t) => {
            _ = Arguments::add(params, t);
        }
        FieldValue::U64(v) => {
            let v = *v as i64;
            _ = Arguments::add(params, v);
        }
        FieldValue::Null => {
            _ = Arguments::add(params, "NULL");
        }
        FieldValue::NotSet => (),
        FieldValue::Failable { field, error } => {
            if error.is_some() {
                return Err(anyhow::anyhow!(error.clone().unwrap()));
            }
            build_field_value_to_args(&field, params)?
        }
    }

    Ok(())
}
