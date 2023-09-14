use crate::app::{
    model::{
        app::{AppEntity, AppService},
        company::{CompanyEntity, CompanyService},
        role::{RoleEntity, RoleService, ROLE_ADMIN},
        role_user::RoleUserService,
        sys_admin::SysAdminService,
    },
    Config, DirtyBase,
};
use busybody::Service;
use dirtybase_db::entity::user::UserEntity;
use fama::PipeContent;

pub(crate) async fn execute() {
    _ = fama::Pipeline::pass(NewSysAdminData::default())
        .through_fn(find_or_create_admin_user)
        .await
        .through_fn(add_user_to_system_wide_admin)
        .await
        .through_fn(create_default_company)
        .await
        .through_fn(create_default_app_add_user)
        .await
        .confirm();
}

#[derive(Clone)]
struct NewSysAdminData {
    user: Option<UserEntity>,
    company: Option<CompanyEntity>,
    company_app: Option<AppEntity>,
    company_app_roles: Option<RoleEntity>,
}

impl Default for NewSysAdminData {
    fn default() -> Self {
        Self {
            user: None,
            company: None,
            company_app: None,
            company_app_roles: None,
        }
    }
}

#[busybody::async_trait]
impl busybody::Injectable for NewSysAdminData {
    async fn inject(c: &busybody::ServiceContainer) -> Self {
        c.proxy_value().unwrap_or_default()
    }
}

async fn find_or_create_admin_user(
    app: Service<DirtyBase>,
    mut new_admin_data: NewSysAdminData,
    mut pipe: PipeContent,
) -> Option<PipeContent> {
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
        if !created {
            log::info!("System admin already exist");
            pipe.stop_the_flow();
        } else {
            new_admin_data.user = Some(user);
            pipe.store(new_admin_data);
            log::info!("System admin created");
        }
    } else {
        pipe.stop_the_flow();
    }

    Some(pipe)
}

async fn add_user_to_system_wide_admin(
    new_admin_data: NewSysAdminData,
    pipe: PipeContent,
    sys_admin_service: SysAdminService,
) -> Option<PipeContent> {
    if new_admin_data.user.is_some() {
        if sys_admin_service
            .add_user(new_admin_data.user.as_ref().unwrap().id.as_ref().unwrap())
            .await
            .is_err()
        {
            pipe.stop_the_flow();
            log::error!("could not add user to system wide admin");
        }
    }
    None
}

async fn create_default_company(
    mut new_admin_data: NewSysAdminData,
    company_service: CompanyService,
    mut new_company: CompanyEntity,
    pipe: PipeContent,
    config: Config,
) -> Option<PipeContent> {
    if new_admin_data.user.is_some() {
        let user = new_admin_data.user.as_ref().clone().unwrap();
        new_company.name = Some(config.app_name().clone());
        new_company.description = Some("This is the core/main company".into());

        if let Ok(company) = company_service
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
    app_service: AppService,
    role_service: RoleService,
    role_user_service: RoleUserService,
    config: Config,
) -> Option<PipeContent> {
    if new_admin_data.user.is_some() && new_admin_data.company.is_some() {
        let company = new_admin_data.company.as_ref().unwrap().clone();
        let user = new_admin_data.user.as_ref().unwrap().clone();

        let mut app_entity = app_service.new_app();
        app_entity.name = Some(format!("{} app", &config.app_name()));
        app_entity.core_company_id = Some(company.id.unwrap());
        app_entity.is_system_app = Some(true);
        app_entity.description = Some("This is the core/main application".into());

        if let Ok(app) = app_service.create(app_entity, user.clone()).await {
            if let Ok(roles) = role_service
                .create_defaults(app.clone(), user.clone())
                .await
            {
                for a_role in &roles {
                    if a_role.name.as_ref().unwrap() == ROLE_ADMIN {
                        let mut role_user = role_user_service.new_role_user();
                        role_user.core_app_role_id = Some(a_role.id.as_ref().unwrap().into());
                        role_user.core_user_id = Some(user.id.as_deref().unwrap().into());

                        let _ = role_user_service.create(role_user, user.clone()).await;
                        break;
                    }
                }
            }
        }
    }

    None
}
