use std::{marker::PhantomData, sync::Arc};

use dirtybase_db::{base::manager::Manager, TableEntityTrait};

pub(crate) struct TableRepo<T: TableEntityTrait> {
    table: String,
    manager: Arc<Manager>,
    phantom: PhantomData<T>,
}

impl<T: TableEntityTrait> TableRepo<T> {
    pub(crate) fn new(manager: Arc<Manager>) -> Self {
        Self {
            table: T::table_name().to_string(),
            phantom: PhantomData,
            manager,
        }
    }

    pub(crate) async fn get(&self) -> Result<Option<Vec<T>>, ::anyhow::Error> {
        self.manager
            .select_from_table(&self.table, |q| {})
            .fetch_all_to()
            .await
    }
}
