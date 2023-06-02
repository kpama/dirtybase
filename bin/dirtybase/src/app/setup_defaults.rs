use super::dirtybase::DirtyBase;

pub async fn setup_default_entities(app: &DirtyBase) {
    let config = app.config();
    let result = app
        .user_service()
        .create_admin_user(
            &config.admin_user(),
            &config.admin_email(),
            &config.admin_password(),
        )
        .await;

    if let Ok((created, user)) = result {
        if created {
            // 1. create company
            // 1.1 create company's default app
            // 1.1.1 add user to the app role as "admin"
            // 1.2 add user as the core_user and creator
        }
    }
}
