use dirtybase_db::types::{ColumnAndValue, FromColumnAndValue, ToColumnAndValue};
use dirtybase_db_macro::DirtyTable;

#[tokio::main]
async fn main() {
    //...
}

#[derive(Debug, Default, Clone)]
struct Config {
    title: String,
    times: u64,
}

impl FromColumnAndValue for Config {
    fn from_column_value(_column_and_value: ColumnAndValue) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        Ok(Self::default())
    }
}

impl ToColumnAndValue for Config {
    fn to_column_value(&self) -> Result<ColumnAndValue, anyhow::Error> {
        Ok(ColumnAndValue::new())
    }
}

#[derive(Debug, Clone, Default, DirtyTable)]
struct Something {
    id: Option<u64>,
    #[dirty(flatten)]
    inner: Config,
}
