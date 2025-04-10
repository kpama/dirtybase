mod models;
mod setup;

use std::{result, sync::Arc};

use dirtybase_helper::ulid::Ulid;
use setup::*;

use dirtybase_contract::db_contract::{
    TableEntityTrait,
    relations::{
        BelongsTo, BelongsToMany, HasMany, HasManyThrough, HasOne, MorphOneOfMany, MorphOneToMany,
        RelationMany, RelationOne, RelationQueryBuilder,
    },
};
use dirtybase_db::{
    config::ConnectionConfig,
    connector::{
        mariadb::make_mariadb_manager, postgres::make_postgres_manager, sqlite::make_sqlite_manager,
    },
};
use models::{
    Address, Company, Customer, Image, Inventory, Invoice, Product, SalesOrder, Warehouse,
};
use rand::Rng;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    // sqlite
    // let base_config = ConnectionConfig {
    //     url: "packages/app/examples/work/data/database.db".to_string(),
    //     ..Default::default()
    // };

    // let manager = make_sqlite_manager(base_config).await;

    // mariadb
    let base_config = ConnectionConfig {
        url: "mariadb://root:dbpassword@db/work".to_string(),
        kind: "mariadb".into(),
        ..Default::default()
    };

    let manager = make_mariadb_manager(base_config).await;

    // postgres
    // let base_config = BaseConfig {
    //     url: "postgres://dbuser:dbpassword@postgres/work".to_string(),
    //     kind: dirtybase_db::base::schema::DatabaseKind::Postgres,
    //     ..Default::default()
    // };

    // let manager = make_postgres_manager(base_config).await;

    create_tables(&manager).await;
    seed_tables(&manager).await;

    let result = manager
        .select_from::<Company>(|q| {
            q.select_table::<Company>();
            q.select_multiple(Address::table_columns());
        })
        .fetch_all_to::<Company>()
        .await;
    let company_count = match result {
        Ok(Some(c)) => c.len(),
        _ => 0,
    };
    println!("company count: {}", company_count);

    // let customer_repo = CustomerRepository::new(manager.clone());
    // let mut builder = customer_repo.builder();

    // //--
    let mut rng = rand::rng();
    let customer = if let Ok(Some(c)) = manager
        .select_from_table(Customer::table_name(), |q| {
            q.eq(
                Customer::col_name_for_internal_id(),
                rng.random_range(1..=200),
            );
        })
        .fetch_one_to::<Customer>()
        .await
    {
        c
    } else {
        Customer::default()
    };

    println!("customer returned: {:?}", &customer.id);

    // // --- example of has many
    let mut has_many = HasMany::<Customer, SalesOrder>::new(manager.clone());
    has_many.owner_key(customer.id.clone());
    println!("total sales order: {}", has_many.count().await);

    if let Ok(Some(list)) = has_many.get().await {
        if let Some(o) = list.first() {
            // --- belongs to example
            let mut belongs_to = BelongsTo::<SalesOrder, Customer>::new(manager.clone());
            belongs_to.parent_key(o.customer_id.clone());
            println!("customer via sales order: {:?}", belongs_to.one().await);

            // --- has one example
            let mut has_one = HasOne::<SalesOrder, Invoice>::new(manager.clone());
            has_one.parent_key(o.id.clone());
            let invoice = has_one.one().await;
            println!("Sales order {:?} invoice = {:#?}", &o.id, invoice);
        }
    }

    // --- belongs to many
    let mut product_id = "".to_string();
    if let Ok(Some(product_instance)) = manager
        .query_builder::<Product>(Product::table_name())
        .one()
        .await
    {
        let mut belongs_to_many =
            BelongsToMany::<Product, Inventory, Warehouse>::new(manager.clone());
        belongs_to_many.constrain_key(product_instance.id.clone());

        if let Ok(Some(w)) = belongs_to_many.pivots().await {
            println!("product inventories: {:#?}", w.len());
        }
        if let Ok(Some(w)) = belongs_to_many.get().await {
            println!("product warehouses: {:#?}", w.len());
        }
        product_id = product_instance.id.as_str().to_string();
    }

    // --- morph one to one
    let mut morph_one_to_one = MorphOneToMany::<Product, Image>::new(
        manager.clone(),
        &Image::col_name_for_imageable_id(),
        &Image::col_name_for_imageable_type(),
    );

    morph_one_to_one.constrain_key(product_id.as_str());

    match morph_one_to_one.one().await {
        Err(e) => println!("product image error: {:?}", e.to_string()),
        Ok(Some(i)) => println!("product image: {}", i.id),
        _ => (),
    }

    // --- morph one to many
    let mut morph_one_to_many = MorphOneToMany::<Product, Image>::new(
        manager.clone(),
        &Image::col_name_for_imageable_id(),
        &Image::col_name_for_imageable_type(),
    );
    morph_one_to_many.constrain_key(product_id.as_str());

    match morph_one_to_many.get().await {
        Err(e) => println!("product images error: {:?}", e),
        Ok(Some(i)) => println!("product images count: {}", i.len()),
        _ => (),
    }

    // -- morph one of many: latest
    let mut morph_one_of_many = MorphOneOfMany::<Product, Image>::new(
        manager.clone(),
        &Image::col_name_for_imageable_id(),
        &Image::col_name_for_imageable_type(),
    );

    morph_one_of_many.parent_key(&product_id);

    // -- latest
    println!(
        "latest image for product: {:#?}",
        morph_one_of_many.latest_of_many(None).await
    );

    // -- oldest
    println!(
        "oldest image for product: {:#?}",
        morph_one_of_many.oldest_of_many(None).await
    );

    // --- warehouse products
    if let Ok(Some(warehouse_instance)) = manager
        .query_builder::<Warehouse>(Warehouse::table_name())
        .one()
        .await
    {
        let mut has_many_through =
            HasManyThrough::<Warehouse, Inventory, Product>::new(manager.clone());
        has_many_through.parent_key(warehouse_instance.id.clone());

        if let Ok(Some(w)) = has_many_through.pivots().await {
            println!("warehouse product inventories: {}", w.len());
        }
        if let Ok(Some(w)) = has_many_through.get().await {
            println!("warehouse products count: {}", w.len());
        }
    }
    // --- ends

    Ok(())
}
