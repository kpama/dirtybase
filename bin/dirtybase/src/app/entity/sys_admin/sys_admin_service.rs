use super::{SysAdminEntity, SysAdminRepository};
use anyhow::anyhow;

pub struct SysAdminService {
    sys_admin_repo: SysAdminRepository,
}

impl SysAdminService {
    pub fn new(sys_admin_repo: SysAdminRepository) -> Self {
        Self { sys_admin_repo }
    }

    pub fn sys_admin_repo(&self) -> &SysAdminRepository {
        &self.sys_admin_repo
    }

    pub fn sys_admin_repo_mut(&mut self) -> &mut SysAdminRepository {
        &mut self.sys_admin_repo
    }

    pub fn new_sys_admin(&self) -> SysAdminEntity {
        SysAdminEntity::new()
    }

    pub async fn add_user(&mut self, user_id: &str) -> Result<SysAdminEntity, anyhow::Error> {
        let mut sys_admin = self.new_sys_admin();
        sys_admin.core_user_id = Some(user_id.into());
        self.create(sys_admin).await
    }

    pub async fn remove_user(&mut self, user_id: &str) -> Result<bool, anyhow::Error> {
        self.sys_admin_repo.delete(user_id).await
    }

    pub async fn create(
        &mut self,
        sys_admin: SysAdminEntity,
    ) -> Result<SysAdminEntity, anyhow::Error> {
        if sys_admin.core_user_id.is_none() {
            return Err(anyhow!("Sys admin permission requires a user"));
        }

        self.sys_admin_repo.create(sys_admin).await
    }
}

#[busybody::async_trait]
impl busybody::Injectable for SysAdminService {
    async fn inject(ci: &busybody::ServiceContainer) -> Self {
        let repo = ci.provide::<SysAdminRepository>().await;

        Self::new(repo)
    }
}
