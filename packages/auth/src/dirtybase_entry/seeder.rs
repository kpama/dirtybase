pub(crate) mod create_auth_users_seeder;

use dirtybase_contract::db_contract::SeederRegisterer;

pub(crate) async fn register_seeders() {
    SeederRegisterer::register("create_auth_actors", |manager, context| {
        Box::pin(async move { create_auth_users_seeder::seed(manager, context).await })
    })
    .await;
    //
}
