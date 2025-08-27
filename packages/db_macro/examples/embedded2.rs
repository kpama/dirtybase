use dirtybase_db::types::ToColumnAndValue;
use dirtybase_db_macro::{DirtyEmbedded, DirtyTable};

#[tokio::main]
async fn main() {
    let s = Something {
        id: Some(43),
        inner: Config {
            title: "foo title".to_string(),
            times: 25,
        },
    };
    let d = serde_json::to_string_pretty(&s.to_column_value().unwrap().to_field_value());
    _ = dbg!(d);
}

#[derive(Debug, Default, Clone, DirtyEmbedded)]
struct Config {
    title: String,
    times: u64,
}

#[derive(Debug, Clone, Default, DirtyTable)]
#[dirty(no_timestamp, no_soft_delete)]
struct Something {
    id: Option<u64>,
    #[dirty(embedded)]
    inner: Config,
}
