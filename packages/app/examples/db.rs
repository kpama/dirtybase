use dirtybase_db::{
    config::ConnectionConfig,
    connector::postgres::make_postgres_manager,
    types::{JsonField, OptionalDateTimeField, OptionalStringField},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    dirtybase_db::setup_handlers().await;

    let base_config = ConnectionConfig {
        url: "postgres://dbuser:dbpassword@postgres/dirtybase".to_string(),
        ..Default::default()
    };

    let manager = make_postgres_manager(base_config).await;
    let result = manager
        .select_from::<SessionTable>(|q| {
            q.eq(
                SessionTable::col_name_for_id(),
                "9528b376c621442edcabd7959b7e52ec",
            );
        })
        .first()
        .await;

    dbg!("{:?}", result);

    Ok(())
}

#[derive(Debug, dirtybase_db_macro::DirtyTable, Default, Clone)]
#[dirty(table = "sessions")]
pub struct SessionTable {
    id: OptionalStringField,
    data: JsonField,
    // data: HashMap<String, String>,
    created_at: OptionalDateTimeField,
    updated_at: OptionalDateTimeField,
}
