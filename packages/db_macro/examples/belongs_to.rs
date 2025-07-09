use dirtybase_db::{
    TableEntityTrait, base::manager::Manager, connector::sqlite::make_sqlite_in_memory_manager,
};
use dirtybase_db_macro::DirtyTable;

#[tokio::main]
async fn main() {
    let manager = make_sqlite_in_memory_manager().await;
    setup_db(&manager).await;

    let mut child_repo = ChildRepo::new(&manager);
    println!("{:#?}", child_repo.with_family().get().await);
}

#[derive(Debug, Default, Clone, DirtyTable)]
struct Family {
    id: Option<i64>,
    name: String,
    #[dirty(rel(kind = "has_many"))]
    children: Vec<Child>,
}

#[derive(Debug, Default, Clone, DirtyTable)]
struct Child {
    id: Option<i64>,
    name: String,
    family_id: i64,
    #[dirty(rel(kind = "belongs_to", scope = "pub(crate)"))]
    family: Family,
}

async fn setup_db(manager: &Manager) {
    create_tables(manager).await;
    seed_tables(manager).await;
}

async fn create_tables(manager: &Manager) {
    //
    _ = manager
        .create_table_schema(Family::table_name(), |table| {
            table.id(None);
            table.string(Family::col_name_for_name());
        })
        .await;

    _ = manager
        .create_table_schema(Child::table_name(), |table| {
            table.id(None);
            table.string(Child::col_name_for_name());
            table.id_table_fk::<Family>(true);
        })
        .await;
}

async fn seed_tables(manager: &Manager) {
    for f in 1..=5 {
        _ = manager
            .insert(
                Family::table_name(),
                Family {
                    name: format!("family {}", f),
                    ..Default::default()
                },
            )
            .await;

        for c in 1..=3 {
            _ = manager
                .insert(
                    Child::table_name(),
                    Child {
                        name: format!("child {} for family {}", c, f),
                        family_id: f,
                        ..Default::default()
                    },
                )
                .await;
        }
    }
}
