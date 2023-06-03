use super::{dirtybase::DirtyBase, entity::company::CompanyEntity};

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
            let mut company = CompanyEntity::new();

            company.name = Some(config.app_name().clone());
            company.description = Some("This is the core/main company".into());

            if let Ok(company) = app
                .company_service()
                .create(company, user.clone(), user.clone())
                .await
            {
                // 1.1 create company's default app
                let mut app_entity = app.app_service().new_app();
                let company_copy = company.clone();

                app_entity.name = Some(format!("{} app", &config.app_name()));
                app_entity.company_id = Some(company_copy.id.unwrap());
                app_entity.is_system_app = Some(true);
                app_entity.description = Some("This is the core/main application".into());

                let result = app.app_service().create(app_entity, user.clone()).await;

                // 1.1.1 add user to the app role as "admin"
                dbg!(&result);
            }

            // 1.2 add user as the core_user and creator
        }
    }
}
