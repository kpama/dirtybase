use dirtybase_contract::{config::DirtyConfig, user::model::UserTrait};
use dirtybase_db::{
    TableEntityTrait, config::ConnectionConfig, connector::sqlite::make_sqlite_manager_from_set,
};
use dirtybase_user::UserEntity;

#[tokio::main]
async fn main() {
    let config_set = ConnectionConfig::set_from(&DirtyConfig::default()).await;
    let manager = make_sqlite_manager_from_set(config_set).await;
    let has_user_table = manager.has_table("users").await;
    let user_entity = UserEntity::default();

    manager.insert(UserEntity::table_name(), user_entity).await;
    if let Ok(Some(users)) = manager
        .select_from::<UserEntity>(|_| {})
        .fetch_all_to::<UserEntity>()
        .await
    {
        for a_user in users {
            println!("user id: {}", a_user.id());
        }
    }

    println!("has_user_table? {}", has_user_table);
}
