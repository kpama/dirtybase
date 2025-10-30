mod models;
mod setup;

use std::collections::HashMap;

use setup::*;

use dirtybase_db::{config::ConnectionConfig, connector::sqlite::make_sqlite_manager};
use tracing::Level;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .try_init()
        .expect("could not setup tracing");

    // sqlite
    let base_config = ConnectionConfig {
        url: "packages/app/examples/work/data/database.db".to_string(),
        ..Default::default()
    };

    let manager = make_sqlite_manager(base_config).await;

    let result = manager
        .transaction(|manager| {
            Box::pin(async move {
                let mut row = HashMap::new();
                row.insert("name".to_string(), "Customer32".into());
                if let Ok(Some(c)) = manager
                    .select_from_table("customers", |qb| {
                        qb.lock_for_update();
                        qb.is_eq("internal_id", 1);
                    })
                    .fetch_one()
                    .await
                {
                    let result = manager
                        .update("customers", row, |qb| {
                            qb.is_eq("internal_id", c.get("internal_id").cloned().unwrap());
                        })
                        .await?;
                    println!("{:#?}", c);
                }
                Ok(())
            })
        })
        .await;

    println!("Done, success: {:?}", result);
    return Ok(());

    // mariadb
    // let base_config = ConnectionConfig {
    //     url: "mariadb://root:dbpassword@db/work".to_string(),
    //     kind: "mariadb".into(),
    //     ..Default::default()
    // };

    // let manager = make_mariadb_manager(base_config).await;

    // postgres
    // let base_config = ConnectionConfig {
    //     url: "postgres://dbuser:dbpassword@postgres/work".to_string(),
    //     kind: "postgres".into(),
    //     ..Default::default()
    // };

    // let manager = make_postgres_manager(base_config).await;

    _ = create_tables(&manager).await;
    seed_tables(&manager).await;

    Ok(())
}
