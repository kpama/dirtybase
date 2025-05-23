use dirtybase_app::setup;
use dirtybase_auth::helpers::get_auth_storage;
use dirtybase_contract::auth_contract::AuthUserPayload;
use dirtybase_db::types::ArcUuid7;
use tracing::Level;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::ERROR)
        .try_init()
        .expect("could not setup tracing");

    let app_service = setup().await.unwrap();
    app_service.init().await;

    let global_context = app_service.global_context().await;
    let storage = get_auth_storage(global_context.clone(), None)
        .await
        .unwrap();

    let id = ArcUuid7::try_from("0195a437-7c59-7471-8c95-c05670e65df7").unwrap();
    if let Ok(Some(existing)) = storage.find_by_id(id).await {
        println!("user already exist: {:?}", existing);
    } else {
        let mut payload = AuthUserPayload::default();
        payload.username = Some("admin".to_string());
        payload.email = Some("example@yahoo.com".to_string());
        payload.password = Some("password".to_string());
        payload.rotate_salt = true;
        if let Ok(x) = storage.store(payload).await {
            println!("x: {:?}", x);
        }
    }

    println!(">>>>>>>>>> completed <<<<<<<<<<<<<");
}
