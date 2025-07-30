use std::{
    fmt::Debug,
    sync::{atomic::AtomicI64, Arc},
};

use crate::db_contract::{
    event::SchemeWroteEvent,
    field_values::FieldValue,
    types::{ColumnAndValue, FromColumnAndValue, ToColumnAndValue},
    DatabaseKindPoolCollection, TableModel,
};

use super::{
    query::{EntityQueryBuilder, QueryBuilder},
    schema::{DatabaseKind, SchemaManagerTrait, SchemaWrapper},
    table::TableBlueprint,
};
use anyhow::{Ok, Result};
use orsomafo::Dispatchable;

#[derive(Clone)]
pub struct Manager {
    connections: Arc<DatabaseKindPoolCollection>,
    kind: DatabaseKind,
    write_is_sticky: bool,
    sticky_duration: i64,
    is_writable: bool,
    last_write_ts: Arc<AtomicI64>,
}

impl Debug for Manager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "db manager: {}", &self.kind)
    }
}

// TODO: add: first or create
// TODO: add: update or create

impl Manager {
    pub fn new(
        connections: Arc<DatabaseKindPoolCollection>,
        kind: DatabaseKind,
        write_is_sticky: bool,
        sticky_duration: i64,
        is_writable: bool,
    ) -> Self {
        Self {
            connections,
            kind,
            write_is_sticky,
            sticky_duration,
            is_writable,
            last_write_ts: Arc::default(),
        }
    }

    // pub fn inner_ref(&self) -> Box<dyn SchemaManagerTrait> {
    //     self.write_schema_manager().as_ref()
    // }

    // Get a table or view for querying
    pub fn select_from_table<F>(&self, table: &str, callback: F) -> SchemaWrapper
    where
        F: FnOnce(&mut QueryBuilder),
    {
        let mut query_builder =
            QueryBuilder::new(table, super::query::QueryAction::Query { columns: None });
        callback(&mut query_builder);

        SchemaWrapper {
            query_builder,
            inner: self.read_schema_manager(),
        }
    }

    pub fn select_from<T>(&self, callback: impl FnOnce(&mut QueryBuilder)) -> SchemaWrapper
    where
        T: TableModel,
    {
        self.select_from_table(T::table_name(), callback)
    }

    pub fn table(&self, table: &str) -> QueryBuilder {
        QueryBuilder::new(table, super::query::QueryAction::Query { columns: None })
    }

    pub fn select_table<T>(&self) -> QueryBuilder
    where
        T: TableModel,
    {
        self.table(T::table_name())
    }

    pub fn query_builder<T: FromColumnAndValue + Send + Sync + 'static>(
        &self,
        table: &str,
    ) -> EntityQueryBuilder<T> {
        EntityQueryBuilder::new(self.table(table), self.read_schema_manager())
    }

    pub fn execute_query(&self, query_builder: QueryBuilder) -> SchemaWrapper {
        SchemaWrapper {
            query_builder,
            inner: self.read_schema_manager(),
        }
    }

    // Create a new table
    pub async fn create_table_schema(
        &self,
        name: &str,
        callback: impl FnOnce(&mut TableBlueprint),
    ) -> Result<(), anyhow::Error> {
        if !self.has_table(name).await? {
            let mut table = self.write_schema_manager().fetch_table_for_update(name);
            table.set_is_new(true);

            callback(&mut table);
            self.write_schema_manager().apply(table).await?;
            self.dispatch_written_event();

            return Ok(());
        }
        Err(anyhow::anyhow!("{} already exist", name))
    }

    // Get an existing table for updating
    pub async fn update_table_schema(
        &self,
        name: &str,
        callback: impl FnOnce(&mut TableBlueprint),
    ) -> Result<(), anyhow::Error> {
        if self.has_table(name).await? {
            let mut table = self.write_schema_manager().fetch_table_for_update(name);
            table.set_is_new(false);

            callback(&mut table);
            self.write_schema_manager().apply(table).await?;
            self.dispatch_written_event();
            return Ok(());
        }
        Err(anyhow::anyhow!("{} does not exist", name))
    }

    // Create a new view
    pub async fn create_view_from_table(
        &self,
        name: &str,
        from_table: &str,
        callback: impl FnOnce(&mut QueryBuilder),
    ) -> Result<(), anyhow::Error> {
        let mut query = QueryBuilder::new(
            from_table,
            super::query::QueryAction::Query { columns: None },
        );
        callback(&mut query);
        let mut table = self.write_schema_manager().fetch_table_for_update(name);
        table.view_query = Some(query);
        self.write_schema_manager().apply(table).await?;
        self.dispatch_written_event();
        Ok(())
    }

    pub async fn insert<CV: ToColumnAndValue>(&self, table_name: &str, record: CV) -> Result<()> {
        self.insert_multi(table_name, vec![record]).await
    }

    pub async fn insert_into<T>(&self, record: impl ToColumnAndValue) -> Result<()>
    where
        T: TableModel,
    {
        self.insert(T::table_name(), record).await
    }

    pub async fn insert_ref<CV: ToColumnAndValue>(
        &self,
        table_name: &str,
        record: &CV,
    ) -> Result<()> {
        self.insert_multi(table_name, vec![record.to_column_value()?])
            .await
    }

    pub async fn insert_multi<I: ToColumnAndValue, R: IntoIterator<Item = I>>(
        &self,
        table_name: &str,
        rows: R,
    ) -> Result<()> {
        self.create_insert_query(table_name, rows, false).await
    }

    /// Insert row gracefully ignore insert duplicates
    pub async fn soft_insert<I: ToColumnAndValue>(&self, table_name: &str, row: I) -> Result<()> {
        self.create_insert_query(table_name, vec![row], true).await
    }

    /// Insert rows gracefully ignore insert duplicates
    pub async fn soft_insert_multi<I: ToColumnAndValue, R: IntoIterator<Item = I>>(
        &self,
        table_name: &str,
        rows: R,
    ) -> Result<()> {
        self.create_insert_query(table_name, rows, true).await
    }

    pub async fn upsert<I: ToColumnAndValue>(
        &self,
        table_name: &str,
        row: I,
        update: &[&str],
        unique: &[&str],
    ) -> Result<()> {
        self.upsert_multi(table_name, vec![row], update, unique)
            .await
    }

    pub async fn upsert_multi<R: IntoIterator<Item = I>, I: ToColumnAndValue>(
        &self,
        table_name: &str,
        rows: R,
        update: &[&str],
        unique: &[&str],
    ) -> Result<()> {
        let query = QueryBuilder::new(
            table_name,
            super::query::QueryAction::Upsert {
                rows: rows.into_iter().flat_map(|r| r.to_column_value()).collect(),
                to_update: update
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>(),
                unique: unique
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>(),
            },
        );

        self.write_schema_manager().execute(query).await?;
        self.dispatch_written_event();
        Ok(())
    }

    pub async fn update<R: ToColumnAndValue>(
        &self,
        table_name: &str,
        row: R,
        callback: impl FnOnce(&mut QueryBuilder),
    ) -> Result<()> {
        let mut query = QueryBuilder::new(
            table_name,
            super::query::QueryAction::Update(row.to_column_value()?),
        );
        callback(&mut query);
        self.write_schema_manager().execute(query).await?;
        self.dispatch_written_event();
        Ok(())
    }

    pub async fn update_table<T: TableModel>(
        &self,
        row: impl ToColumnAndValue,
        callback: impl FnOnce(&mut QueryBuilder),
    ) -> Result<()> {
        self.update(T::table_name(), row, callback).await
    }

    pub async fn delete(
        &self,
        table_name: &str,
        callback: impl FnOnce(&mut QueryBuilder),
    ) -> Result<()> {
        let mut query = QueryBuilder::new(table_name, super::query::QueryAction::Delete);
        callback(&mut query);
        self.write_schema_manager().execute(query).await?;
        self.dispatch_written_event();
        Ok(())
    }

    pub async fn delete_from_table<T>(&self, callback: impl FnOnce(&mut QueryBuilder)) -> Result<()>
    where
        T: TableModel,
    {
        self.delete(T::table_name(), callback).await
    }

    pub async fn transaction<R>(&self, _callback: impl FnMut(&mut QueryBuilder) -> R) -> Result<R> {
        todo!()
    }

    pub async fn has_table(&self, name: &str) -> Result<bool, anyhow::Error> {
        self.read_schema_manager().has_table(name).await
    }

    pub async fn drop_table(&self, table_name: &str) -> Result<(), anyhow::Error> {
        let _query = QueryBuilder::new(table_name, super::query::QueryAction::DropTable);
        self.write_schema_manager().drop_table(table_name).await
    }

    pub async fn rename_table(&self, old: &str, new: &str) -> Result<()> {
        self.write_schema_manager().rename_table(old, new).await?;
        self.dispatch_written_event();
        Ok(())
    }

    pub async fn drop_column(&self, table: &str, column: &str) -> Result<()> {
        self.write_schema_manager()
            .drop_column(table, column)
            .await?;
        self.dispatch_written_event();
        Ok(())
    }

    pub async fn rename_column(&self, table: &str, old: &str, new: &str) -> Result<()> {
        self.write_schema_manager()
            .rename_column(table, old, new)
            .await?;
        self.dispatch_written_event();
        Ok(())
    }

    pub fn read_schema_manager(&self) -> Box<dyn SchemaManagerTrait + Send> {
        self.create_schema_manager(false)
    }

    pub fn write_schema_manager(&self) -> Box<dyn SchemaManagerTrait + Send> {
        self.create_schema_manager(true)
    }

    async fn create_insert_query<I: ToColumnAndValue, R: IntoIterator<Item = I>>(
        &self,
        table_name: &str,
        rows: R,
        do_soft_insert: bool,
    ) -> Result<()> {
        let query = QueryBuilder::new(
            table_name,
            super::query::QueryAction::Create {
                rows: rows.into_iter().flat_map(|r| r.to_column_value()).collect(),
                do_soft_insert,
            },
        );

        self.write_schema_manager().execute(query).await?;
        self.dispatch_written_event();
        Ok(())
    }

    pub async fn raw_insert<V: Into<FieldValue>>(
        &self,
        sql: &str,
        row: Vec<V>,
    ) -> Result<bool, anyhow::Error> {
        let result = self
            .write_schema_manager()
            .raw_insert(sql, row.into_iter().map(|v| v.into()).collect())
            .await;
        if result.is_ok() {
            self.dispatch_written_event();
        }
        result
    }

    pub async fn raw_update<V: Into<FieldValue>>(
        &self,
        sql: &str,
        params: Vec<V>,
    ) -> Result<u64, anyhow::Error> {
        let result = self
            .write_schema_manager()
            .raw_update(sql, params.into_iter().map(|v| v.into()).collect())
            .await;
        if result.is_ok() {
            self.dispatch_written_event();
        }

        result
    }

    pub async fn raw_delete<P: Into<FieldValue>>(
        &self,
        sql: &str,
        params: Vec<P>,
    ) -> Result<u64, anyhow::Error> {
        let result = self
            .write_schema_manager()
            .raw_delete(sql, params.into_iter().map(|v| v.into()).collect())
            .await;
        if result.is_ok() {
            self.dispatch_written_event();
        }

        result
    }

    pub async fn raw_select<P: Into<FieldValue>>(
        &self,
        sql: &str,
        params: Vec<P>,
    ) -> Result<Vec<ColumnAndValue>, anyhow::Error> {
        self.read_schema_manager()
            .raw_select(sql, params.into_iter().map(|v| v.into()).collect())
            .await
    }

    pub async fn raw_statement(&self, sql: &str) -> Result<bool, anyhow::Error> {
        let result = self.write_schema_manager().raw_statement(sql).await;

        if result.is_ok() {
            self.dispatch_written_event();
        }

        result
    }

    pub fn is_writable(&self) -> bool {
        self.is_writable
    }

    pub async fn close(self) {
        for (_, collection) in self.connections.iter() {
            for pool in collection.values() {
                pool.close().await;
            }
        }
    }

    pub fn db_kind(&self) -> &DatabaseKind {
        &self.kind
    }

    fn create_schema_manager(&self, for_write: bool) -> Box<dyn SchemaManagerTrait + Send> {
        match self.connections.get(&self.kind) {
            Some(pool) => {
                if for_write {
                    if let Some(write_pool) = pool.get(&super::schema::ClientType::Write) {
                        log::trace!("Using {:?}'s write pool for next query", &self.kind);
                        write_pool.schema_manger()
                    } else {
                        log::error!(target: "dirtybase_db", "could not create a write schema manager for: {:?}", self.kind);
                        panic!(
                            "could not create a write schema manager for: {:?}",
                            self.kind
                        );
                    }
                } else {
                    // Sticky is on?
                    let ts = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs() as i64;
                    let last_ts = self
                        .last_write_ts
                        .load(std::sync::atomic::Ordering::Relaxed);
                    if self.write_is_sticky && ts - last_ts <= self.sticky_duration {
                        return self.create_schema_manager(true);
                    }

                    if let Some(read_pool) = pool.get(&super::schema::ClientType::Read) {
                        log::trace!("Using {:?}'s read pool for next query", &self.kind);
                        read_pool.schema_manger()
                    } else {
                        log::trace!("Using {:?}'s write pool for next read query", &self.kind);
                        self.create_schema_manager(true)
                    }
                }
            }
            None => {
                log::error!(target: "dirtybase_db", "could not get pool manager for: {:?}", self.kind);
                panic!("could not get pool manager for: {:?}", self.kind);
            }
        }
    }

    fn dispatch_written_event(&self) {
        let ts = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs() as i64;
        self.last_write_ts
            .swap(ts, std::sync::atomic::Ordering::Relaxed);
        (SchemeWroteEvent::new(self.kind.clone(), ts)).dispatch_event();
    }
}
