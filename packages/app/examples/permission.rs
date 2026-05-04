use dirtybase_contract::auth_contract::Actor;
use dirtybase_db::base::{manager::Manager, paginate_builder::PaginateBuilder};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        // .with_max_level(Level::DEBUG)
        .try_init()
        .expect("could not setup tracing");

    let app_service = dirtybase_app::setup()
        .await
        .expect("could not setup application");

    app_service.init().await;

    let context = app_service.global_context().await;

    let manager: Manager = context.get().await.expect("could not get db manager");

    let mut page = PaginateBuilder::new("username", 0, 25);
    page.add_order("id", dirtybase_db::base::order_by_builder::Direction::ASC);
    loop {
        println!("{:?}", serde_json::to_string(&page));

        let (result, p, _) = manager
            .select_from::<Actor>(|_| {
                //
            })
            .paginate_to::<Actor>(page)
            .await
            .parts();

        if result.is_err() {
            break;
        }
        if let Ok(rows) = result {
            println!("result: {:#?}", &rows);
            if rows.is_empty() {
                break;
            }
        }

        if let Some(p) = p {
            page = p
        } else {
            break;
        }
    }
}
