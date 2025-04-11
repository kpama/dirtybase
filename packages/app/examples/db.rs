use dirtybase_db::{
    TableEntityTrait,
    config::ConnectionConfig,
    connector::postgres::make_postgres_manager,
    types::IntegerField,
};
use dirtybase_db_macro::DirtyTable;
use tracing::Level;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .try_init()
        .expect("could not setup tracing");

    dirtybase_db::setup_handlers().await;

    let base_config = ConnectionConfig {
        url: "postgres://dbuser:dbpassword@postgres/dirtybase".to_string(),
        kind: "postgres".into(),
        ..Default::default()
    };
    // let base_config = ConnectionConfig {
    //     url: "mariadb://root:dbpassword@mariadb/dirtybase".to_string(),
    //     kind: "mariadb".into(),
    //     ..Default::default()
    // };

    let manager = make_postgres_manager(base_config).await;
    // let manager = make_mariadb_manager(base_config).await;
    // let manager = make_sqlite_in_memory_manager().await;

    let first = Score {
        user_id: 1,
        points: 5,
    };
    let second = Score {
        user_id: 2,
        points: 6,
    };
    let result = manager
        .upsert_multi(
            Score::table_name(),
            [first, second],
            &["points"],
            &["user_id"],
        )
        .await;
    println!("insert result: {:?}", result);

    Ok(())
}

#[derive(Debug, Clone, Default, DirtyTable)]
struct Score {
    user_id: IntegerField,
    points: IntegerField,
}
