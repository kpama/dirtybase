use dirtybase_db::{
    TableModel,
    base::manager::Manager,
    connector::sqlite::make_sqlite_in_memory_manager,
    types::{ColumnAndValue, CreatedAtField, DeletedAtField, UpdatedAtField},
};
use dirtybase_db_macro::DirtyTable;
use dirtybase_helper::time::current_datetime;
use tracing::Level;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .try_init()
        .expect("could not setup tracing");
    //
    let manager = make_sqlite_in_memory_manager().await;
    setup_db(&manager).await;

    let mut data = ColumnAndValue::new();
    data.insert(
        Child::col_name_for_deleted_at().to_string(),
        dirtybase_helper::time::current_datetime().into(),
    );
    let mut to_delete = Vec::new();
    for i in 1..rand::random_range(1..10) {
        to_delete.push(i);
    }
    _ = manager
        .update_table::<Child>(data, |query| {
            query.is_in(Child::col_name_for_id(), &to_delete);
        })
        .await;

    let mut family_repo = FamilyRepo::new(&manager);
    println!(
        "{:#?}",
        family_repo.with_trashed_only_children().get().await
    );
}

#[derive(Debug, Default, DirtyTable)]
struct Family {
    id: Option<i64>,
    name: String,
    #[dirty(rel(kind = "has_many"))]
    children: Vec<Child>,
    created_at: CreatedAtField,
    updated_at: UpdatedAtField,
    deleted_at: DeletedAtField,
}

#[derive(Debug, Default, DirtyTable)]
#[dirty(table = "family_children")]
struct Child {
    id: Option<i64>,
    name: String,
    family_id: i64,
    created_at: DeletedAtField,
    updated_at: UpdatedAtField,
    deleted_at: DeletedAtField,
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
            table.soft_deletable();
            table.timestamps();
        })
        .await;

    _ = manager
        .create_table_schema(Child::table_name(), |table| {
            table.id(None);
            table.string(Child::col_name_for_name());
            table.id_table_fk::<Family>(true);
            table.soft_deletable();
            table.timestamps();
        })
        .await;
}

async fn seed_tables(manager: &Manager) {
    for f in 1..=5 {
        _ = manager
            .insert(
                Family::table_name(),
                Family {
                    name: format!("family {f}"),
                    created_at: current_datetime().into(),
                    ..Default::default()
                },
            )
            .await;

        for c in 1..=rand::random_range(1..rand::random_range(1..3)) {
            _ = manager
                .insert(
                    Child::table_name(),
                    Child {
                        name: format!("child {c} for family {f}"),
                        family_id: f,
                        created_at: current_datetime().into(),
                        ..Default::default()
                    },
                )
                .await;
        }
    }
}
