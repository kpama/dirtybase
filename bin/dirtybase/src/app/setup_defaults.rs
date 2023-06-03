use super::{
    dirtybase::DirtyBase,
    entity::{company::CompanyEntity, role::ROLE_ADMIN},
};

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
            // 1. add user to system wild  admin list
            let _ = app
                .sys_admin_service()
                .add_user(&user.clone().id.unwrap())
                .await;

            // 2. create default company
            let mut company = CompanyEntity::new();

            company.name = Some(config.app_name().clone());
            company.description = Some("This is the core/main company".into());

            if let Ok(company) = app
                .company_service()
                .create(company, user.clone(), user.clone())
                .await
            {
                // 2.1 create company's default app
                let mut app_entity = app.app_service().new_app();
                let company_copy = company.clone();

                app_entity.name = Some(format!("{} app", &config.app_name()));
                app_entity.company_id = Some(company_copy.id.unwrap());
                app_entity.is_system_app = Some(true);
                app_entity.description = Some("This is the core/main application".into());

                if let Ok(default_app) = app.app_service().create(app_entity, user.clone()).await {
                    match app
                        .role_service()
                        .create_defaults(default_app.clone(), user.clone())
                        .await
                    {
                        Ok(roles) => {
                            for a_role in &roles {
                                if a_role.name.as_ref().unwrap() == ROLE_ADMIN {
                                    // 2.1.1 add user to the app role as "admin"
                                    let mut role_user = app.role_user_service().new_role_user();
                                    role_user.core_app_role_id =
                                        Some(a_role.id.as_ref().unwrap().into());
                                    role_user.core_user_id =
                                        Some(user.id.as_deref().unwrap().into());
                                    let _ = app
                                        .role_user_service()
                                        .create(role_user, user.clone())
                                        .await;
                                }
                            }
                        }
                        Err(_) => (), // Err(e) => log::error!("could not create default roles: {}", e.to_string());
                    }
                }
            }
        }
    }
}
