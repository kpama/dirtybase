use dirtybase_db::{
    TableEntityTrait, base::manager::Manager, connector::sqlite::make_sqlite_in_memory_manager,
};
use dirtybase_db_macro::DirtyTable;
use rand::distr::SampleString;

#[tokio::main]
async fn main() {
    let manager = make_sqlite_in_memory_manager().await;
    setup_db(&manager).await;
    let mut machanic_repo = MachanicRepo::new(&manager);
    println!("{:#?}", machanic_repo.with_car().with_owner().get().await);
}

#[derive(Debug, Default, Clone, DirtyTable)]
struct Machanic {
    id: Option<i64>,
    name: String,
    #[dirty(rel(
      kind = "has_one_through",
      pivot = Car,
      pivot_through_key = id,
      through_key = car_id 
    ))]
    owner: Option<Owner>,
    #[dirty(rel(kind = has_one))]
    car: Option<Car>
}

#[derive(Debug, Default, Clone, DirtyTable)]
struct Car {
    id: Option<i64>,
    model: String,
    machanic_id: i64,
}

#[derive(Debug, Default, Clone, DirtyTable)]
struct Owner {
    id: Option<i64>,
    name: String,
    car_id: i64,
}

async fn setup_db(manager: &Manager) {
    create_tables(manager).await;
    seed_db(manager).await;
}

async fn create_tables(manager: &Manager) {
    _ = manager
        .create_table_schema(Machanic::table_name(), |table| {
            table.id(None);
            table.string(Machanic::col_name_for_name());
        })
        .await;

    _ = manager
        .create_table_schema(Car::table_name(), |table| {
            table.id(None);
            table.string(Car::col_name_for_model());
            table.id_table_fk::<Machanic>(true);
        })
        .await;

    _ = manager
        .create_table_schema(Owner::table_name(), |table| {
            table.id(None);
            table.string(Owner::col_name_for_name());
            table.id_table_fk::<Car>(true);
        })
        .await;
}

async fn seed_db(manager: &Manager) {
    for _ in 1..=5 {
        _ = manager
            .insert(
                Machanic::table_name(),
                Machanic {
                    name: rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 10),
                    ..Default::default()
                },
            )
            .await;
    }

    for m in 1..=5 {
        _ = manager
            .insert(
                Car::table_name(),
                Car {
                    model: rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 10),
                    machanic_id: m,
                    ..Default::default()
                },
            )
            .await;
    }

    for c in 1..=5 {
        _ = manager
            .insert(
                Owner::table_name(),
                Owner {
                    name: rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 10),
                    car_id: c,
                    ..Default::default()
                },
            )
            .await;
    }
}
