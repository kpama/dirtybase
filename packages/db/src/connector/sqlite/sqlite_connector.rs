use crate::base::{
    column::{ColumnBlueprint, ColumnDefault, ColumnType},
    index::IndexType,
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
    base::aggregate::Aggregate,
    query_column::{QueryColumn, QueryColumnName},
};
use futures::stream::TryStreamExt;
use sqlx::{
    Arguments, Column, Pool, Row, Sqlite, SqliteTransaction, TypeInfo,
    sqlite::{SqliteArguments, SqliteRow},
    types::chrono,
};
use std::{collections::HashMap, sync::Arc};

const LOG_TARGET: &str = "sqlite_db_driver";
pub const SQLITE_KIND: &str = "sqlite";

pub struct SqliteSchemaManager {
    db_pool: Arc<Pool<Sqlite>>,
    trans: Option<SqliteTransaction<'static>>,
}

impl SqliteSchemaManager {
    pub fn new(db_pool: Arc<Pool<Sqlite>>) -> Self {
        Self {
            db_pool,
            trans: None,
        }
    }

    pub fn new_trans(db_pool: Arc<Pool<Sqlite>>, trans: SqliteTransaction<'static>) -> Self {
        Self {
            db_pool,
            trans: Some(trans),
        }
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
    async fn has_table(&mut self, name: &str) -> Result<bool, anyhow::Error> {
        let query = "SELECT name FROM sqlite_master WHERE name = ?";

        let result = sqlx::query(query)
            .bind(name)
            .map(|_row| true)
            .fetch_one(self.db_pool.as_ref())
            .await;

        Ok(result.unwrap_or_default())
    }

    async fn begin(&mut self) -> Result<Box<dyn SchemaManagerTrait>, anyhow::Error> {
        match self.db_pool.begin().await {
            Ok(trans) => Ok(Box::new(Self::new_trans(self.db_pool.clone(), trans))),
            Err(e) => Err(e.into()),
        }
    }

    async fn commit(&mut self) -> Result<(), anyhow::Error> {
        if let Some(trans) = self.trans.take() {
            if let Err(e) = trans.commit().await {
                return Err(e.into());
            }
        }

        Ok(())
    }

    async fn rollback(&mut self) -> Result<(), anyhow::Error> {
        if let Some(trans) = self.trans.take() {
            if let Err(e) = trans.rollback().await {
                return Err(e.into());
            }
        }

        Ok(())
    }

    async fn stream_result(
        &mut self,
        query_builder: &QueryBuilder,
        sender: tokio::sync::mpsc::Sender<ColumnAndValue>,
    ) -> Result<(), anyhow::Error> {
        let mut params = SqliteArguments::default();
        let statement = self.build_query(query_builder, &mut params)?;

        let query = sqlx::query_with(&statement, params);

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

    async fn drop_table(&mut self, name: &str) -> Result<(), anyhow::Error> {
        if self.has_table(name).await? {
            let query = QueryBuilder::new(name, QueryAction::DropTable);
            return self.execute(query).await;
        }
        Ok(())
    }

    async fn apply(&mut self, table: TableBlueprint) -> Result<(), anyhow::Error> {
        self.do_apply(table).await
    }

    async fn execute(&mut self, query: QueryBuilder) -> anyhow::Result<()> {
        self.do_execute(query).await
    }

    async fn fetch_all(
        &mut self,
        query_builder: &QueryBuilder,
    ) -> Result<Option<Vec<HashMap<String, FieldValue>>>, anyhow::Error> {
        let mut results = Vec::new();

        let mut params = SqliteArguments::default();
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
        &mut self,
        query_builder: &QueryBuilder,
    ) -> Result<Option<ColumnAndValue>, anyhow::Error> {
        let mut params = SqliteArguments::default();

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

    async fn raw_insert(&mut self, sql: &str, row: Vec<FieldValue>) -> Result<bool, anyhow::Error> {
        let mut query = sqlx::query(sql);
        for field in row {
            query = query.bind(field.to_string());
        }
        match query.execute(self.db_pool.as_ref()).await {
            Ok(_) => Ok(true),
            Err(e) => Err(e.into()),
        }
    }

    async fn raw_update(
        &mut self,
        sql: &str,
        params: Vec<FieldValue>,
    ) -> Result<u64, anyhow::Error> {
        let mut query = sqlx::query(sql);
        for p in params {
            query = query.bind(p.to_string());
        }

        match query.execute(self.db_pool.as_ref()).await {
            Ok(v) => Ok(v.rows_affected()),
            Err(e) => Err(anyhow::anyhow!(e)),
        }
    }

    async fn raw_delete(
        &mut self,
        sql: &str,
        params: Vec<FieldValue>,
    ) -> Result<u64, anyhow::Error> {
        self.raw_update(sql, params).await
    }

    async fn raw_select(
        &mut self,
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

    async fn raw_statement(&mut self, sql: &str) -> Result<bool, anyhow::Error> {
        let query = sqlx::query(sql);

        match query.execute(self.db_pool.as_ref()).await {
            Ok(_v) => Ok(true),
            Err(e) => Err(e.into()),
        }
    }
}

impl SqliteSchemaManager {
    async fn do_apply(&mut self, table: TableBlueprint) -> Result<(), anyhow::Error> {
        return if table.view_query.is_some() {
            // working with view table
            self.create_or_replace_view(table).await
        } else {
            // working with real table
            self.apply_table_changes(table).await
        };
    }

    async fn do_execute(&mut self, query: QueryBuilder) -> anyhow::Result<()> {
        let mut params = SqliteArguments::default();

        let mut sql;
        match query.action() {
            QueryAction::Create {
                rows,
                do_soft_insert,
            } => {
                sql = format!(
                    "INSERT {} INTO '{}' ",
                    if *do_soft_insert { "OR IGNORE" } else { "" },
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
                            .map(|e| format!("`{e}`"))
                            .collect::<Vec<String>>()
                            .join(",")
                    );

                    let mut update_values = Vec::new();
                    for entry in to_update {
                        update_values.push(format!("`{entry}` = `excluded`.`{entry}`"));
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
                sql = format!("UPDATE `{}` SET ", query.table());
                for entry in columns.iter().enumerate() {
                    if entry.0 > 0 {
                        sql = format!("{}, `{}` = ? ", sql, *entry.1);
                    } else {
                        sql = format!("{} `{}` = ? ", sql, *entry.1);
                    }
                }

                sql = format!("{} {}", sql, self.build_where_clauses(&query, &mut params)?);
            }
            QueryAction::Delete => {
                sql = format!("DELETE FROM '{0}' ", query.table());
                // joins
                sql = format!("{} {}", sql, self.build_join(&query, &mut params)?);
                // where
                sql = format!("{} {}", sql, self.build_where_clauses(&query, &mut params)?);
            }
            QueryAction::DropTable => {
                sql = format!("DROP TABLE IF EXISTS '{}';", query.table());
            }
            QueryAction::RenameColumn { old, new } => {
                let table = query.table();
                sql = format!("ALTER TABLE '{table}' RENAME COLUMN '{old}' TO '{new}'");
            }
            QueryAction::RenameTable(new) => {
                let table = query.table();
                sql = format!("ALTER TABLE '{table}' RENAME TO '{new}'");
            }
            QueryAction::DropColumn(column) => {
                let table = query.table();
                sql = format!("ALTER TABLE '{table}' DROP COLUMN '{column}'");
            }
            _ => {
                sql = "".into();
            }
        }

        let result = if let Some(mut trans) = self.trans.take() {
            let result = sqlx::query_with(&sql, params).execute(&mut *trans).await;
            if result.is_ok() {
                if let Err(e) = trans.commit().await {
                    tracing::error!(target: LOG_TARGET, "committing error: {}", &e);
                    return Err(e.into());
                }
            } else {
                if let Err(e) = trans.rollback().await {
                    tracing::error!(target: LOG_TARGET, "rolling back error: {}", &e);
                    return Err(e.into());
                }
            }

            result
        } else {
            sqlx::query_with(&sql, params)
                .execute(self.db_pool.as_ref())
                .await
        };

        match result {
            Ok(r) => {
                log::debug!(target: LOG_TARGET,"{} result: {:#?}", query.action(), r);
                Ok(())
            }
            Err(e) => {
                log::error!(target: LOG_TARGET, "{} failed: {}", query.action(), e);
                Err(anyhow!(e))
            }
        }
    }

    async fn create_or_replace_view(&self, table: TableBlueprint) -> Result<(), anyhow::Error> {
        if let Some(query) = &table.view_query {
            let mut params = SqliteArguments::default();
            let sql = self.build_query(query, &mut params)?;

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
                        &error
                    );
                    return Err(anyhow::anyhow!(error));
                }
            }
        }

        Ok(())
    }

    async fn apply_table_changes(&self, table: TableBlueprint) -> Result<(), anyhow::Error> {
        let mut foreign = Vec::new();
        let columns: Vec<String> = table
            .columns()
            .iter()
            .map(|column| self.create_column(column, &mut foreign, table.is_new()))
            .collect();

        let mut query = if table.is_new() {
            format!("CREATE TABLE `{}`", &table.name)
        } else {
            format!("ALTER TABLE `{}`", &table.name)
        };

        if table.is_new() {
            query = if foreign.is_empty() {
                format!("{} ({})", query, columns.join(","))
            } else {
                format!("{} ({}, {})", query, columns.join(","), foreign.join(","))
            }
        } else {
            query = if foreign.is_empty() {
                format!("{} ADD {}", query, columns.join(","))
            } else {
                format!(
                    "{} ADD COLUMN {} {}",
                    query,
                    columns.join(","),
                    foreign.join(",")
                )
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
                        log::error!("could not create table index: {}", &e);
                        return Err(anyhow::anyhow!("could not create table index: {}", &e));
                    }
                }
            }
        }

        Ok(())
    }

    fn create_column(
        &self,
        column: &ColumnBlueprint,
        foreign: &mut Vec<String>,
        is_new_table: bool,
    ) -> String {
        let mut entry = format!("`{}`", &column.name);
        let mut the_type = " ".to_owned();

        // column type
        match column.column_type {
            ColumnType::AutoIncrementId => {
                the_type.push_str("INTEGER");
                foreign.push(format!("PRIMARY KEY('{}' AUTOINCREMENT)", &column.name));
            }
            ColumnType::Boolean => the_type.push_str("BOOLEAN"),
            ColumnType::Char(length) => the_type.push_str(&format!("VARCHAR({length})")),
            ColumnType::Datetime => the_type.push_str("datetime"),
            ColumnType::Date => the_type.push_str("DATE"),
            ColumnType::Timestamp => the_type.push_str("timestamp"),
            ColumnType::Float => the_type.push_str("double"),
            ColumnType::Integer => the_type.push_str("INTEGER"),
            ColumnType::Json => the_type.push_str("json"),
            ColumnType::Number => the_type.push_str("double"),
            ColumnType::Binary => the_type.push_str("BLOB"),
            ColumnType::String(length) => {
                let q = format!("VARCHAR({length})");
                the_type.push_str(q.as_str());
            }
            ColumnType::Text => the_type.push_str("TEXT"),
            ColumnType::Uuid => the_type.push_str("BLOB"),
            ColumnType::Enum(ref opt) => {
                if column.check.is_none() {
                    let list = opt
                        .iter()
                        .map(|e| format!("'{e}'"))
                        .collect::<Vec<String>>()
                        .join(",");
                    the_type.push_str(&format!(
                        "varchar(255) CONSTRAINT {0}_chk check (\"{0}\" in ({1}))",
                        column.name, list
                    ));
                } else {
                    the_type.push_str("varchar(255)"); // The check will be added below
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

        // primary key
        if column.is_primary {
            foreign.push(format!("PRIMARY KEY('{}')", &column.name));
        }

        // column default
        if let Some(default) = &column.default {
            the_type.push_str(" DEFAULT ");
            match default {
                // ColumnDefault::CreatedAt => (), // the_type.push_str("CURRENT_TIMESTAMP"),
                ColumnDefault::Custom(d) => the_type.push_str(&format!("'{d}'")),
                ColumnDefault::EmptyArray => the_type.push_str("'[]'"),
                ColumnDefault::EmptyObject => the_type.push_str("'{}'"),
                ColumnDefault::EmptyString => the_type.push_str("''"),
                ColumnDefault::Uuid => (), // the_type.push_str("GUID()"),
                ColumnDefault::Ulid => (),
                // ColumnDefault::UpdatedAt => (), // the_type.push_str("CURRENT_TIMESTAMP"),
                ColumnDefault::Zero => the_type.push('0'),
            };
        }

        // column relationship
        if let Some(relationship) = &column.relationship {
            let mut f = "".to_string();
            let st = if is_new_table {
                format!(
                    "FOREIGN KEY (`{}`) REFERENCES `{}` (`{}`)",
                    &column.name,
                    &relationship.table(),
                    &relationship.column()
                )
            } else {
                format!(
                    "REFERENCES `{}` (`{}`)",
                    &relationship.table(),
                    &relationship.column()
                )
            };
            f.push_str(&st);
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
                        .map(|e| format!("'{e}'"))
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
        params: &mut SqliteArguments,
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
                sql = format!("{sql} *") // Select all columns by default
            }
        }

        // join fields
        if let Some(joins) = query.joins() {
            for a_join in joins.values() {
                if let Some(columns) = a_join.select_columns() {
                    let mut col_names = Vec::new();
                    for a_field in columns {
                        col_names.push(self.column_to_string(a_field, params)?);
                    }
                    sql = format!("{}, {}", sql, col_names.join(","));
                }
            }
        }

        // from
        sql = format!("{} FROM '{}'", sql, query.table());

        // joins
        sql = format!("{} {}", sql, self.build_join(query, params)?);

        // wheres
        sql = format!("{} {}", sql, self.build_where_clauses(query, params)?);

        // group by

        // order by
        if let Some(order) = self.build_order_by(query) {
            sql = format!("{sql} {order}");
        }

        // limit
        if let Some(limit) = query.limit_by() {
            sql = format!("{sql} {limit}");
        }

        // TODO: offset

        // Not supported in sqlite
        // if query.is_lock_for_update() {
        //     sql = format!("{sql} FOR UPDATE");
        // }

        Ok(sql)
    }

    fn build_join(
        &self,
        query: &QueryBuilder,
        _params: &mut SqliteArguments,
    ) -> Result<String, anyhow::Error> {
        let mut sql = "".to_string();
        if let Some(joins) = query.joins() {
            for a_join in joins.values() {
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
        params: &mut SqliteArguments,
    ) -> Result<String, anyhow::Error> {
        let mut wheres = "".to_owned();
        for where_join in query.where_clauses() {
            wheres = where_join.as_clause(
                &wheres,
                &self.transform_condition(where_join.condition(), params)?,
            );
        }

        if !wheres.is_empty() {
            wheres = format!("WHERE {wheres}");
        }

        Ok(wheres)
    }

    fn transform_condition(
        &self,
        condition: &Condition,
        params: &mut SqliteArguments,
    ) -> Result<String, anyhow::Error> {
        let placeholder = match condition.value() {
            QueryValue::SubQuery(sub) => self.build_query(sub, params)?,
            QueryValue::ColumnName(name) => name.clone(),
            _ => {
                self.transform_value(condition.value(), params)?;
                if *condition.operator() == Operator::In || *condition.operator() == Operator::NotIn
                {
                    let length = match &condition.value() {
                        QueryValue::Field(FieldValue::Array(v)) => v.len(),
                        _ => 1,
                    };

                    let mut placeholder = Vec::new();
                    placeholder.resize(length, "?");
                    placeholder.join(",")
                } else {
                    "?".to_owned()
                }
            }
        };

        Ok(condition
            .operator()
            .as_clause(condition.column(), &placeholder))
    }

    fn transform_value(
        &self,
        value: &QueryValue,
        params: &mut SqliteArguments,
    ) -> Result<(), anyhow::Error> {
        match value {
            QueryValue::SubQuery(q) => {
                self.build_query(q, params)?;
            }
            QueryValue::Field(field) => self.field_value_to_args(field, params)?,
            QueryValue::Null => (),          // `is null` or `is not null`
            QueryValue::ColumnName(_) => (), // Does not require an entry into the params,
        }
        Ok(())
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
                    let v = row.try_get::<Vec<u8>, &str>(col.name());
                    if let Ok(v) = v {
                        this_row.insert(col.name().to_string(), FieldValue::Binary(v));
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
                        target: LOG_TARGET,
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
        params: &mut SqliteArguments,
    ) -> Result<(), anyhow::Error> {
        build_field_value_to_args(field, params)
    }

    fn build_insert_data(
        &self,
        params: &mut SqliteArguments,
        rows: &[ColumnAndValue],
        mut sql: String,
    ) -> Result<String, anyhow::Error> {
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
                .map(|e| format!("`{e}`"))
                .collect::<Vec<String>>()
                .join(",");

            sql = format!("{sql} ({columns}) VALUES ");

            for a_row in rows.iter().enumerate() {
                for col in &keys {
                    let field = a_row.1.get(col).unwrap();
                    self.field_value_to_args(field, params)?;
                }
                let separator = if a_row.0 > 0 { "," } else { "" };

                sql = format!("{} {} ({})", sql, separator, &placeholders);
            }
        }

        Ok(sql)
    }

    fn column_to_string(
        &self,
        column: &QueryColumn,
        params: &mut SqliteArguments,
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
                        format!("`{tbl}`.`{n}`")
                    } else {
                        n.clone()
                    };
                    if alias.is_empty() {
                        Ok(format!("({aggregate}({full_name})) as '{full_name}'"))
                    } else {
                        Ok(format!("{aggregate}({full_name}) as '{alias}'"))
                    }
                }
                QueryColumnName::SubQuery(query) => {
                    let sql = self.build_query(query, params)?;
                    if alias.is_empty() {
                        Ok(sql)
                    } else {
                        Ok(format!("({aggregate}({sql})) as '{alias}'"))
                    }
                }
            };
        }
        match column.name() {
            QueryColumnName::Name(n) => {
                let full_name = if let Some(tbl) = column.table() {
                    format!("`{tbl}`.`{n}`")
                } else {
                    n.clone()
                };

                if alias.is_empty() {
                    Ok(full_name.to_string())
                } else {
                    Ok(format!("{full_name} as '{alias}'"))
                }
            }
            QueryColumnName::SubQuery(query) => {
                let sql = self.build_query(query, params)?;
                if alias.is_empty() {
                    Ok(sql)
                } else {
                    Ok(format!("({sql}) as '{alias}'"))
                }
            }
        }
    }
}

fn build_field_value_to_args(
    field: &FieldValue,
    params: &mut SqliteArguments,
) -> Result<(), anyhow::Error> {
    match field.clone() {
        // sqlite arguments uses a lifetime
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
                build_field_value_to_args(&entry, params)?;
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
        FieldValue::Failable { field, error } => {
            if error.is_some() {
                return Err(anyhow::anyhow!(error.unwrap()));
            }
            build_field_value_to_args(&field, params)?
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{config::ConnectionConfig, connector::sqlite::sqlite_pool_manager::db_connect};

    use super::*;

    #[tokio::test]
    async fn test_select_builder() {
        let mut query = QueryBuilder::new("foo", QueryAction::Query { columns: None });
        let config = ConnectionConfig::default();
        let pool = db_connect(&config).await;
        let sqlite = SqliteSchemaManager::new(Arc::new(pool.unwrap()));
        let mut params = SqliteArguments::default();

        let mut builder = QueryBuilder::new("inner", QueryAction::Query { columns: None });
        // builder.max_as("point", "user_points");
        builder.is_eq("user", 32);
        let mut col = QueryColumn::from(QueryColumnName::SubQuery(builder));
        col.set_alias("points");
        col.set_aggregate(Aggregate::Count);

        query.is_eq("age", 54);
        query.select(col);

        // use to test generated sql
        println!("{:#?}", sqlite.build_query(&query, &mut params));
        println!("{:#?}", &params)
    }
}
