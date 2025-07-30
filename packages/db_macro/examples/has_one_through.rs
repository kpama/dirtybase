use dirtybase_db::{
    TableModel, base::manager::Manager, connector::sqlite::make_sqlite_in_memory_manager,
};
use dirtybase_db_macro::DirtyTable;
use rand::distr::SampleString;

#[tokio::main]
async fn main() {
    let manager = make_sqlite_in_memory_manager().await;
    setup_db(&manager).await;
    let mut employee_repo = EmployeeRepo::new(&manager);
    println!(
        "{:#?}",
        employee_repo.with_de().with_department().get().await
    );
}

#[derive(Debug, Default, Clone, DirtyTable)]
struct Employee {
    id: Option<i64>,
    name: String,
    #[dirty(rel(
        kind = "has_one_through",
        pivot: DepartmentEmployee
    ))]
    department: Option<Department>,
    #[dirty(rel(kind = has_one ))]
    de: Option<DepartmentEmployee>,
}

#[derive(Debug, Default, Clone, DirtyTable)]
struct Department {
    id: Option<i64>,
    name: String,
}

#[derive(Debug, Default, Clone, DirtyTable)]
struct DepartmentEmployee {
    id: Option<i64>,
    department_id: i64,
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
            table.soft_deletable();
        })
        .await;
    _ = manager
        .create_table_schema(Department::table_name(), |table| {
            table.id(None);
            table.string(Department::col_name_for_name());
            table.soft_deletable();
        })
        .await;
    _ = manager
        .create_table_schema(DepartmentEmployee::table_name(), |table| {
            table.id(None);
            table.id_table_fk::<Department>(true);
            table.id_table_fk::<Employee>(true);
            table.soft_deletable();
            table.unique_index(&[
                Department::foreign_id_column(),
                Employee::foreign_id_column(),
            ]);
        })
        .await;
}

async fn seed_tables(manager: &Manager) {
    for _ in 1..=5 {
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
    }

    for _ in 1..=5 {
        _ = manager
            .insert(
                Department::table_name(),
                Department {
                    name: rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 10),
                    ..Default::default()
                },
            )
            .await;
    }

    for id in 1..=5 {
        _ = manager
            .insert(
                DepartmentEmployee::table_name(),
                DepartmentEmployee {
                    employee_id: id,
                    department_id: rand::random_range(1..=5),
                    ..Default::default()
                },
            )
            .await;
    }
}
