use super::{query::QueryBuilder, save::SaveRecord, schema::SchemaManagerTrait, table::BaseTable};
use sqlx::any::AnyKind;

pub struct Manager {
    schema: Box<dyn SchemaManagerTrait>,
}

impl Manager {
    pub fn new(schema: Box<dyn SchemaManagerTrait>) -> Self {
        Self { schema }
    }

    pub fn db_kind(&self) -> AnyKind {
        self.schema.kind()
    }

    pub fn is_mysql(&self) -> bool {
        self.db_kind() == AnyKind::MySql
    }

    pub fn inner(&mut self) -> &dyn SchemaManagerTrait {
        self.schema.as_mut()
    }

    // Get a table or view for querying
    pub fn table<F>(&mut self, table: &str, callback: F) -> &dyn SchemaManagerTrait
    where
        F: FnMut(&mut QueryBuilder),
    {
        self.tables(vec![table.to_owned()], callback)
    }

    // Get tables or view for querying
    pub fn tables<F>(&mut self, tables: Vec<String>, mut callback: F) -> &dyn SchemaManagerTrait
    where
        F: FnMut(&mut QueryBuilder),
    {
        let mut query = QueryBuilder::new(tables);
        callback(&mut query);
        self.schema.query(query)
    }

    // Create a new table
    pub async fn create(&self, name: &str, mut callback: impl FnMut(&mut BaseTable)) {
        if !self.has_table(name).await {
            let mut table = self.schema.fetch_table_for_update(name);
            table.set_is_new(true);

            callback(&mut table);
            self.schema.commit(table).await;
        }
    }

    // Get an existing table for updating
    pub async fn update(&self, name: &str, mut callback: impl FnMut(&mut BaseTable)) {
        if self.has_table(name).await {
            let mut table = self.schema.fetch_table_for_update(name);
            table.set_is_new(false);

            callback(&mut table);
            self.schema.commit(table).await;
        }
    }

    // Create a new view
    pub async fn view_from_table(
        &self,
        name: &str,
        from_table: &str,
        mut callback: impl FnMut(&mut QueryBuilder),
    ) {
        let mut query = QueryBuilder::new(vec![from_table.to_owned()]);
        callback(&mut query);
        let mut table = self.schema.fetch_table_for_update(name);
        table.view_query = Some(query);
        self.schema.commit(table).await;
    }

    pub fn insert(&self, name: &str) -> SaveRecord {
        SaveRecord::new(name)
    }

    pub async fn has_table(&self, name: &str) -> bool {
        self.schema.has_table(name).await
    }
}
