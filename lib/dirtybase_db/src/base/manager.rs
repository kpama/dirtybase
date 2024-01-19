use std::{collections::HashMap, sync::Arc};

use crate::{
    config::DirtybaseDbConfig,
    event::SchemeWroteEvent,
    field_values::FieldValue,
    query_values::QueryValue,
    types::{ColumnAndValue, IntoColumnAndValue},
    ConnectionsType, LAST_WRITE_TS,
};

use super::{
    query::QueryBuilder,
    schema::{DatabaseKind, SchemaManagerTrait, SchemaWrapper},
    table::BaseTable,
};
use orsomafo::Dispatchable;

pub struct Manager {
    connections: Arc<ConnectionsType>,
    kind: DatabaseKind,
    config: DirtybaseDbConfig,
}

impl Manager {
    pub fn new(
        connections: Arc<ConnectionsType>,
        kind: DatabaseKind,
        config: DirtybaseDbConfig,
    ) -> Self {
        Self {
            connections,
            kind,
            config,
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
        self.select_from_tables(vec![table.to_owned()], callback)
    }

    pub fn table(&self, table: &str) -> QueryBuilder {
        QueryBuilder::new(
            vec![table.to_string()],
            super::query::QueryAction::Query {
                columns: None,
                select_all: true,
            },
        )
    }

    // Get tables or view for querying
    pub fn select_from_tables<F>(&self, tables: Vec<String>, callback: F) -> SchemaWrapper
    where
        F: FnOnce(&mut QueryBuilder),
    {
        let mut query_builder = QueryBuilder::new(
            tables,
            super::query::QueryAction::Query {
                columns: None,
                select_all: true,
            },
        );
        callback(&mut query_builder);

        SchemaWrapper {
            query_builder,
            inner: self.read_schema_manager(),
        }
    }

    // Create a new table
    pub async fn create_table_schema(&self, name: &str, mut callback: impl FnOnce(&mut BaseTable)) {
        if !self.has_table(name).await {
            let mut table = self.write_schema_manager().fetch_table_for_update(name);
            table.set_is_new(true);

            callback(&mut table);
            self.write_schema_manager().apply(table).await;
            self.dispatch_written_event();
        }
    }

    // Get an existing table for updating
    pub async fn update_table_schema(&self, name: &str, mut callback: impl FnOnce(&mut BaseTable)) {
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
        mut callback: impl FnOnce(&mut QueryBuilder),
    ) {
        let mut query = QueryBuilder::new(
            vec![from_table.to_owned()],
            super::query::QueryAction::Query {
                columns: None,
                select_all: false,
            },
        );
        callback(&mut query);
        let mut table = self.write_schema_manager().fetch_table_for_update(name);
        table.view_query = Some(query);
        self.write_schema_manager().apply(table).await;
        self.dispatch_written_event();
    }

    // TODO: Return a result ...
    pub async fn insert<CV: IntoColumnAndValue>(&self, table_name: &str, record: CV) {
        self.insert_multi(table_name, vec![record.into_column_value()])
            .await;
    }

    pub async fn insert_multi(&self, table_name: &str, rows: Vec<ColumnAndValue>) {
        self.create_insert_query(table_name, rows, false).await
    }

    /// Insert a row gracefully ignore insert creates duplicate
    pub async fn soft_insert(&self, table_name: &str, column_and_values: ColumnAndValue) {
        self.create_insert_query(table_name, vec![column_and_values], true)
            .await;
    }

    /// Insert rows gracefully ignore insert duplicates
    pub async fn soft_insert_multi(&self, table_name: &str, rows: Vec<ColumnAndValue>) {
        self.create_insert_query(table_name, rows, true).await
    }

    // TODO: Return a result ...
    pub async fn update(
        &self,
        table_name: &str,
        column_and_values: ColumnAndValue,
        mut callback: impl FnOnce(&mut QueryBuilder),
    ) {
        let mut query = QueryBuilder::new(
            vec![table_name.to_owned()],
            super::query::QueryAction::Update(column_and_values),
        );
        callback(&mut query);
        self.write_schema_manager().execute(query).await;
        self.dispatch_written_event();
    }

    pub async fn delete(&self, table_name: &str, mut callback: impl FnOnce(&mut QueryBuilder)) {
        let mut query = QueryBuilder::new(
            vec![table_name.to_owned()],
            super::query::QueryAction::Delete,
        );
        callback(&mut query);
        self.write_schema_manager().execute(query).await;
        self.dispatch_written_event();
    }

    pub async fn transaction(
        &self,
        table_name: &str,
        mut callback: impl FnOnce(&mut QueryBuilder),
    ) {
        todo!()
    }

    pub async fn has_table(&self, name: &str) -> bool {
        self.read_schema_manager().has_table(name).await
    }

    pub async fn drop_table(&self, table_name: &str) -> bool {
        let query = QueryBuilder::new(
            vec![table_name.to_owned()],
            super::query::QueryAction::DropTable,
        );
        self.write_schema_manager().drop_table(table_name).await
    }

    fn read_schema_manager(&self) -> Box<dyn SchemaManagerTrait + Send> {
        self.create_schema_manager(false)
    }

    fn write_schema_manager(&self) -> Box<dyn SchemaManagerTrait + Send> {
        self.create_schema_manager(true)
    }

    async fn create_insert_query(
        &self,
        table_name: &str,
        rows: Vec<ColumnAndValue>,
        do_soft_insert: bool,
    ) {
        let query = QueryBuilder::new(
            vec![table_name.to_owned()],
            super::query::QueryAction::Create {
                rows,
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
                    let mut sticky = false;
                    let mut sticky_duration = 0;
                    let config = match self.kind {
                        DatabaseKind::Mysql => self.config.mysql_write.as_ref(),
                        DatabaseKind::Postgres => self.config.postgres_write.as_ref(),
                        DatabaseKind::Sqlite => self.config.sqlite_write.as_ref(),
                    };

                    if let Some(conf) = config {
                        sticky = conf.sticky.unwrap_or_default();
                        sticky_duration = conf.sticky_duration.unwrap_or_default();
                    }

                    if sticky && sticky_duration > 0 {
                        if let Some(log) = LAST_WRITE_TS.get() {
                            let now = chrono::Utc::now().timestamp();
                            let mut use_write = false;
                            if let Ok(lock) = log.read() {
                                if let Some(entry) = lock.get(&self.kind) {
                                    use_write = (sticky_duration + entry) >= now;
                                }
                            }
                            if use_write {
                                return self.write_schema_manager();
                            }
                        }
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
        (SchemeWroteEvent::new(self.kind.clone())).dispatch_event();
    }
}
