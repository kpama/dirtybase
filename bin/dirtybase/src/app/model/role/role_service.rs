use super::{RoleEntity, RoleRepository, ROLE_ADMIN, ROLE_USER};
use crate::app::model::app::AppEntity;
use anyhow::anyhow;
use dirtybase_contract::db::{base::helper::generate_ulid, entity::user::UserEntity};

pub struct RoleService {
    role_repo: RoleRepository,
}

impl RoleService {
    pub fn new(role_repo: RoleRepository) -> Self {
        Self { role_repo }
    }

    pub fn role_repo(&self) -> &RoleRepository {
        &self.role_repo
    }

    pub fn role_repo_mut(&mut self) -> &mut RoleRepository {
        &mut self.role_repo
    }

    pub fn new_role(&self) -> RoleEntity {
        RoleEntity::new()
    }

    pub async fn create_defaults(
        &self,
        app: AppEntity,
        blame: UserEntity,
    ) -> Result<Option<Vec<RoleEntity>>, anyhow::Error> {
        if app.id.is_none() {
            return Err(anyhow!("A role must be assigned to an application"));
        }

        let id = app.id.unwrap();
        let mut result = Vec::with_capacity(2);

        // admin role
        let mut admin_role = self.new_role();
        admin_role.name = Some(ROLE_ADMIN.into());
        admin_role.core_app_id = Some(id.clone());

        if let Ok(Some(role)) = self.create(admin_role, blame.clone()).await {
            result.push(role);
        }

        // user role
        let mut user_role = self.new_role();
        user_role.name = Some(ROLE_USER.into());
        user_role.core_app_id = Some(id);

        if let Ok(Some(role)) = self.create(user_role, blame).await {
            result.push(role);
        }

        Ok(Some(result))
    }

    pub async fn create(
        &self,
        mut role: RoleEntity,
        blame: UserEntity,
    ) -> Result<Option<RoleEntity>, anyhow::Error> {
        // TODO: validation
        if role.name.is_none() {
            return Err(anyhow!("A new role requires a name"));
        }

        if role.core_app_id.is_none() {
            return Err(anyhow!("A role must be assigned to an application"));
        }

        if blame.id.is_none() {
            return Err(anyhow!("Role entity requires a user to blame"));
        }

        // prep
        if role.id.is_none() {
            role.id = Some(generate_ulid());
        }
        role.creator_id = Some(blame.id.unwrap());

        return self.role_repo.create(role).await;
    }

    pub async fn update(
        &self,
        mut role: RoleEntity,
        id: &str,
        blame: UserEntity,
    ) -> Result<Option<RoleEntity>, anyhow::Error> {
        // TODO: Validation ....
        if blame.id.is_none() {
            return Err(anyhow!("Role entity requires a user to blame"));
        }

        role.editor_id = Some(blame.id.unwrap());
        self.role_repo.update(id, role).await
    }
}

#[busybody::async_trait]
impl busybody::Injectable for RoleService {
    async fn inject(ci: &busybody::ServiceContainer) -> Self {
        let role_repo = ci.provide::<RoleRepository>().await;

        Self::new(role_repo)
    }
}
