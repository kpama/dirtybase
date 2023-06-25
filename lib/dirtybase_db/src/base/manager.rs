use super::{
    query::QueryBuilder, schema::SchemaManagerTrait, table::BaseTable, types::ColumnAndValue,
};

pub struct Manager {
    schema: Box<dyn SchemaManagerTrait + Send>,
}

impl Manager {
    pub fn new(schema: Box<dyn SchemaManagerTrait + Send>) -> Self {
        Self { schema }
    }

    pub fn inner(&mut self) -> &dyn SchemaManagerTrait {
        self.schema.as_mut()
    }

    // Get a table or view for querying
    pub fn select_from_table<F>(&mut self, table: &str, callback: F) -> &dyn SchemaManagerTrait
    where
        F: FnMut(&mut QueryBuilder),
    {
        self.select_from_tables(vec![table.to_owned()], callback)
    }

    // Get tables or view for querying
    pub fn select_from_tables<F>(
        &mut self,
        tables: Vec<String>,
        mut callback: F,
    ) -> &dyn SchemaManagerTrait
    where
        F: FnMut(&mut QueryBuilder),
    {
        let mut query = QueryBuilder::new(tables, super::query::QueryAction::Query);
        callback(&mut query);
        self.schema.query(query)
    }

    // Create a new table
    pub async fn create_table_schema(&self, name: &str, mut callback: impl FnMut(&mut BaseTable)) {
        if !self.has_table(name).await {
            let mut table = self.schema.fetch_table_for_update(name);
            table.set_is_new(true);

            callback(&mut table);
            self.schema.commit(table).await;
        }
    }

    // Get an existing table for updating
    pub async fn update_table_schema(&self, name: &str, mut callback: impl FnMut(&mut BaseTable)) {
        if self.has_table(name).await {
            let mut table = self.schema.fetch_table_for_update(name);
            table.set_is_new(false);

            callback(&mut table);
            self.schema.commit(table).await;
        }
    }

    // Create a new view
    pub async fn create_view_from_table(
        &self,
        name: &str,
        from_table: &str,
        mut callback: impl FnMut(&mut QueryBuilder),
    ) {
        let mut query = QueryBuilder::new(
            vec![from_table.to_owned()],
            super::query::QueryAction::Query,
        );
        callback(&mut query);
        let mut table = self.schema.fetch_table_for_update(name);
        table.view_query = Some(query);
        self.schema.commit(table).await;
    }

    // TODO: Return a result ...
    pub async fn insert(&self, table_name: &str, column_and_values: ColumnAndValue) {
        let mut query = QueryBuilder::new(
            vec![table_name.to_owned()],
            super::query::QueryAction::Create,
        );
        query.set_multiple(column_and_values);
        self.schema.execute(query).await;
    }

    // TODO: Return a result ...
    pub async fn update(
        &self,
        table_name: &str,
        column_and_values: ColumnAndValue,
        mut callback: impl FnMut(&mut QueryBuilder),
    ) {
        let mut query = QueryBuilder::new(
            vec![table_name.to_owned()],
            super::query::QueryAction::Update,
        );
        query.set_multiple(column_and_values);
        callback(&mut query);
        self.schema.execute(query).await;
    }

    pub async fn delete(&self, table_name: &str, mut callback: impl FnMut(&mut QueryBuilder)) {
        let mut query = QueryBuilder::new(
            vec![table_name.to_owned()],
            super::query::QueryAction::Delete,
        );
        callback(&mut query);
        self.schema.execute(query).await;
    }

    pub async fn has_table(&self, name: &str) -> bool {
        self.schema.has_table(name).await
    }
}
