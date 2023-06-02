use super::dirtybase::DirtyBase;

pub async fn setup_default_entities(app: &DirtyBase) {
    let config = app.config();
    let _ = app
        .user_service()
        .create_admin_user(
            &config.admin_user(),
            &config.admin_email(),
            &config.admin_password(),
        )
        .await;
}
