use std::sync::{atomic::AtomicI64, Arc};

use crate::db::{
    config::ConfigSet,
    event::SchemeWroteEvent,
    field_values::FieldValue,
    types::{ColumnAndValue, FromColumnAndValue, IntoColumnAndValue},
    DatabaseKindPoolCollection,
};

use super::{
    query::{EntityQueryBuilder, QueryBuilder},
    schema::{DatabaseKind, SchemaManagerTrait, SchemaWrapper},
    table::BaseTable,
};
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

impl Manager {
    pub fn new(
        connections: Arc<DatabaseKindPoolCollection>,
        kind: DatabaseKind,
        config_set: ConfigSet,
    ) -> Self {
        let mut write_is_sticky = false;
        let mut sticky_duration = 0;
        let mut is_writable = false;

        if let Some(config) = config_set.get(&super::schema::ClientType::Write) {
            write_is_sticky = config.sticky.clone().unwrap_or_default();
            sticky_duration = config.sticky_duration.clone().unwrap_or_default();
            is_writable = true;
        }

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

    pub fn table(&self, table: &str) -> QueryBuilder {
        QueryBuilder::new(table, super::query::QueryAction::Query { columns: None })
    }

    pub fn query_builder<T: FromColumnAndValue + Send + Sync + 'static>(
        &self,
        table: &str,
    ) -> EntityQueryBuilder<T> {
        EntityQueryBuilder::new(self.table(table), self.read_schema_manager())
    }

    // // Get tables or view for querying
    // pub fn select_from_tables<F>(&self, tables: Vec<String>, callback: F) -> SchemaWrapper
    // where
    //     F: FnOnce(&mut QueryBuilder),
    // {
    //     let mut query_builder =
    //         QueryBuilder::new(tables, super::query::QueryAction::Query { columns: None });
    //     callback(&mut query_builder);

    //     SchemaWrapper {
    //         query_builder,
    //         inner: self.read_schema_manager(),
    //     }
    // }

    pub fn execute_query(&self, query_builder: QueryBuilder) -> SchemaWrapper {
        SchemaWrapper {
            query_builder,
            inner: self.read_schema_manager(),
        }
    }

    // Create a new table
    pub async fn create_table_schema(&self, name: &str, callback: impl FnOnce(&mut BaseTable)) {
        if !self.has_table(name).await {
            let mut table = self.write_schema_manager().fetch_table_for_update(name);
            table.set_is_new(true);

            callback(&mut table);
            self.write_schema_manager().apply(table).await;
            self.dispatch_written_event();
        }
    }

    // Get an existing table for updating
    pub async fn update_table_schema(&self, name: &str, callback: impl FnOnce(&mut BaseTable)) {
        if self.has_table(name).await {
            let mut table = self.write_schema_manager().fetch_table_for_update(name);
            table.set_is_new(false);

            callback(&mut table);
            self.write_schema_manager().apply(table).await;
            self.dispatch_written_event();
        }
    }

    // Create a new view
    pub async fn create_view_from_table(
        &self,
        name: &str,
        from_table: &str,
        callback: impl FnOnce(&mut QueryBuilder),
    ) {
        let mut query = QueryBuilder::new(
            from_table,
            super::query::QueryAction::Query { columns: None },
        );
        callback(&mut query);
        let mut table = self.write_schema_manager().fetch_table_for_update(name);
        table.view_query = Some(query);
        self.write_schema_manager().apply(table).await;
        self.dispatch_written_event();
    }

    // TODO: Return a result ...
    pub async fn insert<CV: IntoColumnAndValue>(&self, table_name: &str, record: CV) {
        self.insert_multi(table_name, vec![record]).await;
    }

    pub async fn insert_multi<I: IntoColumnAndValue, R: IntoIterator<Item = I>>(
        &self,
        table_name: &str,
        rows: R,
    ) {
        self.create_insert_query(table_name, rows, false).await
    }

    /// Insert a row gracefully ignore insert creates duplicate
    pub async fn soft_insert<I: IntoColumnAndValue>(&self, table_name: &str, row: I) {
        self.create_insert_query(table_name, vec![row], true).await;
    }

    /// Insert rows gracefully ignore insert duplicates
    pub async fn soft_insert_multi<I: IntoColumnAndValue, R: IntoIterator<Item = I>>(
        &self,
        table_name: &str,
        rows: R,
    ) {
        self.create_insert_query(table_name, rows, true).await
    }

    // TODO: Return a result ...
    pub async fn update<R: IntoColumnAndValue>(
        &self,
        table_name: &str,
        row: R,
        callback: impl FnOnce(&mut QueryBuilder),
    ) {
        let mut query = QueryBuilder::new(
            table_name,
            super::query::QueryAction::Update(row.into_column_value()),
        );
        callback(&mut query);
        self.write_schema_manager().execute(query).await;
        self.dispatch_written_event();
    }

    // TODO: Return a resut.....
    pub async fn delete(&self, table_name: &str, callback: impl FnOnce(&mut QueryBuilder)) {
        let mut query = QueryBuilder::new(table_name, super::query::QueryAction::Delete);
        callback(&mut query);
        self.write_schema_manager().execute(query).await;
        self.dispatch_written_event();
    }

    pub async fn transaction(&self, _table_name: &str, _callback: impl FnOnce(&mut QueryBuilder)) {
        todo!()
    }

    pub async fn has_table(&self, name: &str) -> bool {
        self.read_schema_manager().has_table(name).await
    }

    pub async fn drop_table(&self, table_name: &str) -> bool {
        let _query = QueryBuilder::new(table_name, super::query::QueryAction::DropTable);
        self.write_schema_manager().drop_table(table_name).await
    }

    pub async fn rename_table(&self, old: &str, new: &str) {
        self.write_schema_manager().rename_table(old, new).await;
        self.dispatch_written_event();
    }

    pub async fn drop_column(&self, table: &str, column: &str) {
        self.write_schema_manager().drop_column(table, column).await;
        self.dispatch_written_event();
    }

    pub async fn rename_column(&self, table: &str, old: &str, new: &str) {
        self.write_schema_manager()
            .rename_column(table, old, new)
            .await;
        self.dispatch_written_event();
    }

    pub fn read_schema_manager(&self) -> Box<dyn SchemaManagerTrait + Send> {
        self.create_schema_manager(false)
    }

    pub fn write_schema_manager(&self) -> Box<dyn SchemaManagerTrait + Send> {
        self.create_schema_manager(true)
    }

    async fn create_insert_query<I: IntoColumnAndValue, R: IntoIterator<Item = I>>(
        &self,
        table_name: &str,
        rows: R,
        do_soft_insert: bool,
    ) {
        let query = QueryBuilder::new(
            table_name,
            super::query::QueryAction::Create {
                rows: rows.into_iter().map(|r| r.into_column_value()).collect(),
                do_soft_insert,
            },
        );

        self.write_schema_manager().execute(query).await;
        self.dispatch_written_event();
    }

    pub async fn raw_insert(
        &self,
        sql: &str,
        values: Vec<Vec<FieldValue>>,
    ) -> Result<bool, anyhow::Error> {
        let result = self.write_schema_manager().raw_insert(sql, values).await;
        if result.is_ok() {
            self.dispatch_written_event();
        }
        result
    }

    pub async fn raw_update(
        &self,
        sql: &str,
        params: Vec<FieldValue>,
    ) -> Result<u64, anyhow::Error> {
        let result = self.write_schema_manager().raw_update(sql, params).await;
        if result.is_ok() {
            self.dispatch_written_event();
        }

        result
    }

    pub async fn raw_delete(
        &self,
        sql: &str,
        params: Vec<FieldValue>,
    ) -> Result<u64, anyhow::Error> {
        let result = self.write_schema_manager().raw_delete(sql, params).await;
        if result.is_ok() {
            self.dispatch_written_event();
        }

        result
    }

    pub async fn raw_select(
        &self,
        sql: &str,
        params: Vec<FieldValue>,
    ) -> Result<Vec<ColumnAndValue>, anyhow::Error> {
        self.read_schema_manager().raw_select(sql, params).await
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

    fn create_schema_manager(&self, for_write: bool) -> Box<dyn SchemaManagerTrait + Send> {
        return match self.connections.get(&self.kind) {
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
                        .load(std::sync::atomic::Ordering::Relaxed)
                        as i64;
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
        };
    }

    fn dispatch_written_event(&self) {
        let ts = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs() as i64;
        self.last_write_ts
            .swap(ts, std::sync::atomic::Ordering::Relaxed);
        (SchemeWroteEvent::new(self.kind.clone(), ts)).dispatch_event();
    }
}
