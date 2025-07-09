use dirtybase_db::{
    TableEntityTrait, base::manager::Manager, connector::sqlite::make_sqlite_in_memory_manager,
};
use dirtybase_db_macro::DirtyTable;
use rand::distr::SampleString;

#[tokio::main]
async fn main() {
    let manager = make_sqlite_in_memory_manager().await;
    setup_db(&manager).await;

    let mut employee_repo = EmployeeRepo::new(&manager);
    println!("{:#?}", employee_repo.with_pin().get().await);
}

#[derive(Debug, Default, Clone, DirtyTable)]
struct Employee {
    id: Option<i64>,
    name: String,
    #[dirty(rel(kind = "has_one",))]
    pin: Option<PinCode>,
}

#[derive(Debug, Default, Clone, DirtyTable)]
struct PinCode {
    id: Option<i64>,
    code: String,
    employee_id: i64,
}

async fn setup_db(manager: &Manager) {
    create_tables(manager).await;
    seed_tables(manager).await;
}

async fn create_tables(manager: &Manager) {
    _ = manager
        .create_table_schema(Employee::table_name(), |table| {
            table.id(None);
            table.string(Employee::col_name_for_name());
        })
        .await;

    _ = manager
        .create_table_schema(PinCode::table_name(), |table| {
            table.id(None);
            table.string(PinCode::col_name_for_code());
            table.id_table_fk::<Employee>(true);
        })
        .await;
}

async fn seed_tables(manager: &Manager) {
    for e in 1..=5 {
        let name = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 10);
        _ = manager
            .insert(
                Employee::table_name(),
                Employee {
                    name,
                    ..Default::default()
                },
            )
            .await;

        _ = manager
            .insert(
                PinCode::table_name(),
                PinCode {
                    code: rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 4),
                    employee_id: e,
                    ..Default::default()
                },
            )
            .await;
    }
}
