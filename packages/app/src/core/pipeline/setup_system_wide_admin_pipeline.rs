use crate::core::{
    model::{
        app_entity::AppEntityService,
        company::{CompanyEntity, CompanyService},
        role::{RoleService, ROLE_ADMIN},
        role_user::RoleUserService,
        sys_admin::SysAdminService,
    },
    App,
};
use busybody::Service;
use dirtybase_user::entity::user::UserEntity;
use fama::PipeContent;

#[derive(Clone, Default)]
pub struct NewSysAdminData {
    user: Option<UserEntity>,
    company: Option<CompanyEntity>,
    // company_app: Option<AppEntity>,
    // company_app_roles: Option<RoleEntity>,
}

#[busybody::async_trait]
impl busybody::Injectable for NewSysAdminData {
    async fn inject(c: &busybody::ServiceContainer) -> Self {
        c.proxy_value().await.unwrap_or_default()
    }
}

#[busybody::async_trait]
impl fama::PipelineBuilderTrait for NewSysAdminData {
    async fn setup_pipeline_builder(
        builder: fama::PipelineBuilder<Self>,
    ) -> fama::PipelineBuilder<Self> {
        builder
            .register(|pipeline| {
                Box::pin(async {
                    pipeline
                        // .through_fn(find_or_create_admin_user)
                        // .await
                        // .through_fn(add_user_to_system_wide_admin)
                        // .await
                        .through_fn(create_default_company)
                        .await
                        .through_fn(create_default_app_add_user)
                        .await
                })
            })
            .await;
        builder
    }
}

async fn find_or_create_admin_user(
    app: Service<App>,
    mut new_admin_data: NewSysAdminData,
    pipe: PipeContent,
) -> Option<PipeContent> {
    let config = app.config();
    let result = app
        .user_service()
        .create_admin_user(
            config.admin_username(),
            config.admin_email(),
            config.admin_password(),
        )
        .await;

    match result {
        Ok(Some((created, user))) => {
            if !created {
                log::info!("System admin already exist");
                pipe.stop_the_flow();
            } else {
                new_admin_data.user = Some(user);
                pipe.store(new_admin_data);
                log::info!("System admin created");
            }
        }
        Ok(None) => {
            log::info!("could not create default user, none returned");
            pipe.stop_the_flow();
        }
        Err(e) => {
            log::info!("could not create default admin user: {:?}", e);
            pipe.stop_the_flow();
        }
    }

    Some(pipe)
}

async fn add_user_to_system_wide_admin(
    new_admin_data: NewSysAdminData,
    pipe: PipeContent,
    sys_admin_service: SysAdminService,
) -> Option<PipeContent> {
    if new_admin_data.user.is_some()
        && sys_admin_service
            .add_user(new_admin_data.user.as_ref().unwrap().id.clone())
            .await
            .is_err()
    {
        pipe.stop_the_flow();
        log::error!("could not add user to system wide admin");
    }
    None
}

async fn create_default_company(
    mut new_admin_data: NewSysAdminData,
    company_service: CompanyService,
    mut new_company: CompanyEntity,
    pipe: PipeContent,
    app: Service<App>,
) -> Option<PipeContent> {
    let config = app.config();
    if new_admin_data.user.is_some() {
        let user = new_admin_data.user.as_ref().unwrap();
        new_company.name = config.app_name().to_string();
        new_company.description = Some("This is the core/main company".into());

        if let Ok(Some(company)) = company_service
            .create(new_company, user.clone(), user.clone())
            .await
        {
            pipe.container().set_type(company.clone());
            new_admin_data.company = Some(company);
            pipe.store(new_admin_data);
        }
    }

    None
}

async fn create_default_app_add_user(
    new_admin_data: NewSysAdminData,
    app_service: AppEntityService,
    role_service: RoleService,
    role_user_service: RoleUserService,
    app: Service<App>,
) -> Option<PipeContent> {
    let config = app.config();
    if new_admin_data.user.is_some() && new_admin_data.company.is_some() {
        let company = new_admin_data.company.as_ref().unwrap().clone();
        let user = new_admin_data.user.as_ref().unwrap().clone();

        let mut app_entity = app_service.new_app();
        app_entity.name = format!("{} app", &config.app_name());
        app_entity.core_company_id = company.id;
        app_entity.is_system_app = true;
        app_entity.description = Some("This is the core/main application".into());

        if let Ok(Some(app)) = app_service.create(app_entity, user.clone()).await {
            if let Ok(Some(roles)) = role_service
                .create_defaults(app.clone(), user.clone())
                .await
            {
                for a_role in &roles {
                    if a_role.name == ROLE_ADMIN {
                        let mut role_user = role_user_service.new_role_user();
                        role_user.core_app_role_id = a_role.id.clone();
                        role_user.core_user_id = user.id.clone();

                        let _ = role_user_service.create(role_user, user.clone()).await;
                        break;
                    }
                }
            }
        }
    }

    None
}
