use anyhow::anyhow;
use dirtybase_db::entity::user::UserEntity;

use super::{RoleUserEntity, RoleUserRepository};

pub struct RoleUserService {
    role_user_repo: RoleUserRepository,
}

impl RoleUserService {
    pub fn new(role_user_repo: RoleUserRepository) -> Self {
        Self { role_user_repo }
    }

    pub fn role_user_repo(&self) -> &RoleUserRepository {
        &self.role_user_repo
    }

    pub fn role_user_repo_mut(&mut self) -> &mut RoleUserRepository {
        &mut self.role_user_repo
    }

    pub fn new_role_user(&self) -> RoleUserEntity {
        RoleUserEntity::new()
    }

    pub async fn create(
        &mut self,
        mut role_user: RoleUserEntity,
        blame: UserEntity,
    ) -> Result<RoleUserEntity, anyhow::Error> {
        // TODO: validation...
        if role_user.core_app_role_id.is_none() || role_user.core_user_id.is_none() {
            return Err(anyhow!("user and app role IDs are always require"));
        }
        if blame.id.is_none() {
            return Err(anyhow!("Role user entity requires a user to blame"));
        }

        role_user.creator_id = Some(blame.id.unwrap());

        self.role_user_repo_mut().create(role_user).await
    }

    pub async fn update(
        &mut self,
        mut role_user: RoleUserEntity,
        blame: UserEntity,
    ) -> Result<RoleUserEntity, anyhow::Error> {
        if role_user.core_app_role_id.is_none() || role_user.core_user_id.is_none() {
            return Err(anyhow!("user and app role IDs are always require"));
        }

        if blame.id.is_none() {
            return Err(anyhow!("Role user entity requires a user to blame"));
        }

        role_user.editor_id = Some(blame.id.unwrap());

        self.role_user_repo_mut().update(role_user).await
    }
}
