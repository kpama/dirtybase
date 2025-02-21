use crate::models::{
    Address, Company, CompanyType, Customer, Foo, Image, Inventory, Invoice, OrderItem, Product,
    SalesOrder, Warehouse,
};
use dirtybase_db::{TableEntityTrait, base::manager::Manager, types::UlidField};
use rand::Rng;

pub async fn seed_tables(manager: &Manager) {
    if let Ok(Some(_)) = manager
        .select_from_table(Customer::table_name(), |q| {
            q.eq(Customer::col_name_for_name(), "Customer1");
        })
        .fetch_one_to::<Customer>()
        .await
    {
        println!("Database has been seeded");
        return; // database has been seeded
    }

    let mut rng = rand::rng();
    println!("Seeding database with mock data");

    // company
    let company_type = [CompanyType::A, CompanyType::B, CompanyType::C];
    for index in 0..=10 {
        let address = Address {
            line_one: format!("{0} Company {0} St", index),
            line_two: format!("Line two for company {}", index),
            state: "Address State".to_string(),
        };
        let company = Company {
            name: format!("Company {}", index),
            address: address.clone(),
            json_address: address,
            company_type: company_type[rng.random_range(0..=2) as usize].clone(),
            logo: b"This is a test".to_vec(),
            ..Default::default()
        };
        manager.insert(Company::table_name(), company).await;
    }

    // warehouse
    for index in 0..=3 {
        let warehouse_name = format!("Warehouse {}", index);
        let warehouse_id = UlidField::new();
        manager
            .insert(
                Warehouse::table_name(),
                Warehouse {
                    internal_id: None,
                    id: warehouse_id,
                    name: warehouse_name,
                },
            )
            .await;
    }

    // product and inventory
    for index in 1..=150 {
        let product_sku = format!("PROD-{}", index);
        let product_id = UlidField::new();
        manager
            .insert(
                Product::table_name(),
                Product {
                    internal_id: None,
                    id: product_id.clone(),
                    sku: product_sku,
                },
            )
            .await;

        // product image
        manager
            .insert(
                Image::table_name(),
                Image::make_imageable::<Product>(
                    &format!("static/images/products/{}/main.jpg", &product_id),
                    &product_id,
                ),
            )
            .await;
        for image_index in 1..rng.random_range(1..=4) {
            manager
                .insert(
                    Image::table_name(),
                    Image::make_imageable::<Product>(
                        &format!(
                            "static/images/products/{}/image{}.jpg",
                            &product_id, image_index
                        ),
                        &product_id,
                    ),
                )
                .await;
        }

        for id in 1..=4 {
            if let Ok(Some(warehouse)) = manager
                .select_from_table(Warehouse::table_name(), |q| {
                    q.eq(Warehouse::col_name_for_internal_id(), id);
                })
                .fetch_one_to::<Warehouse>()
                .await
            {
                manager
                    .insert(
                        Inventory::table_name(),
                        Inventory {
                            warehouse_id: warehouse.id,
                            product_id: product_id.clone(),
                            quantity: rng.random_range(1.0..=6000.0),
                        },
                    )
                    .await;
            }
        }
    }

    // customer
    for index in 1..2001 {
        let customer_name = format!("Customer{}", index);
        let customer_id = UlidField::new();
        manager
            .insert(
                Customer::table_name(),
                Customer {
                    internal_id: None,
                    id: customer_id.clone(),
                    name: customer_name,
                },
            )
            .await;

        // customer image: avatar
        manager
            .insert(
                Image::table_name(),
                Image::make_imageable::<Customer>(
                    &format!("static/images/customers/{}/avatar.jpg", &customer_id),
                    &customer_id,
                ),
            )
            .await;

        // customer sales order, order items and invoice
        for _ in 1..rng.random_range(1..=101) {
            let order_id = UlidField::new();
            manager
                .insert(
                    SalesOrder::table_name(),
                    SalesOrder {
                        customer_id: customer_id.clone(),
                        id: order_id.clone(),
                        ..Default::default()
                    },
                )
                .await;

            manager
                .insert(
                    Invoice::table_name(),
                    Invoice {
                        internal_id: None,
                        id: Default::default(),
                        sales_order_id: order_id.clone(),
                        total: rng.random_range(10.0..=10000.0),
                        paid: rng.random_range(10.0..=10000.0),
                    },
                )
                .await;

            // sales order line item
            let mut items = Vec::new();
            let mut product_lists = Vec::new();
            let total = rng.random_range(1..=15);
            let mut index = 1_usize;
            loop {
                let item_name = format!("Line Item {}", index);
                loop {
                    if let Ok(Some(product)) = manager
                        .select_from_table(Product::table_name(), |q| {
                            q.eq(
                                Product::col_name_for_sku(),
                                format!("PROD-{}", rng.random_range(1..=150)),
                            );
                        })
                        .fetch_one_to::<Product>()
                        .await
                    {
                        if product_lists.contains(product.internal_id.as_ref().unwrap()) {
                            continue;
                        }
                        product_lists.push(product.internal_id.as_ref().unwrap().clone());
                        items.push(OrderItem {
                            id: Default::default(),
                            name: item_name.to_string(),
                            sales_order_id: order_id.clone(),
                            product_id: product.id.clone(),
                            ..Default::default()
                        });
                        index += 1;
                        break;
                    }
                }

                if items.len() >= total {
                    break;
                }
            }

            if items.is_empty() {
                panic!("line item is empty");
            }
            manager.insert_multi(OrderItem::table_name(), items).await;
        }
    }
}

pub async fn create_tables(manager: &Manager) {
    // foo
    manager
        .create_table_schema(Foo::table_name(), |table| {
            table.id_set();
            table.binary(Foo::col_name_for_data());
            table.float(Foo::col_name_for_amount());
            table.integer(Foo::col_name_for_count());
        })
        .await;

    // company
    manager
        .create_table_schema(Company::table_name(), |table| {
            table.id_set();
            table.string(Company::col_name_for_name());
            table.string(Address::col_name_for_line_one());
            table.string(Address::col_name_for_line_two());
            table.string(Address::col_name_for_state());
            table.json(Company::col_name_for_json_address());
            table.enum_(Company::col_name_for_company_type(), &["a", "b", "c"]);
            table.binary(Company::col_name_for_logo());
        })
        .await;
    // customer
    manager
        .create_table_schema(Customer::table_name(), |table| {
            table.id_set();
            table.string(Customer::col_name_for_name());
        })
        .await;
    // product
    manager
        .create_table_schema(Product::table_name(), |table| {
            table.id_set();
            table.string(Product::col_name_for_sku());
        })
        .await;

    // warehouse
    manager
        .create_table_schema(Warehouse::table_name(), |table| {
            table.id_set();
            table.string(Warehouse::col_name_for_name());
        })
        .await;

    // inventory
    manager
        .create_table_schema(Inventory::table_name(), |table| {
            table.ulid_table_fk::<Warehouse>(true);
            table.ulid_table_fk::<Product>(true);
            table.number(Inventory::col_name_for_quantity());
        })
        .await;

    // sales order
    manager
        .create_table_schema(SalesOrder::table_name(), |table| {
            table.id_set();
            table.ulid_table_fk::<Customer>(true);
        })
        .await;
    // order item
    manager
        .create_table_schema(OrderItem::table_name(), |table| {
            table.id_set();
            table.ulid_table_fk::<SalesOrder>(true);
            table.ulid_table_fk::<Product>(true);
            table.string(OrderItem::col_name_for_name());
        })
        .await;

    // image
    manager
        .create_table_schema(Image::table_name(), |table| {
            table.id_set();
            table.string(Image::col_name_for_url());
            table
                .ulid(Image::col_name_for_imageable_id())
                .set_is_nullable(false);
            table
                .string(Image::col_name_for_imageable_type())
                .set_is_nullable(false);
            table.created_at();
        })
        .await;

    // invoice
    manager
        .create_table_schema(Invoice::table_name(), |table| {
            table.id_set();
            table.ulid_table_fk::<SalesOrder>(true);
            table.number(Invoice::col_name_for_total());
            table.number(Invoice::col_name_for_paid());
        })
        .await;
}
