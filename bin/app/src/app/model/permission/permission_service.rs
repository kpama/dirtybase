use super::{permission_entity::PermissionEntity, permission_repository::PermissionRepository};
use anyhow::anyhow;

pub struct PermissionService {
    repo: PermissionRepository,
}

impl PermissionService {
    pub fn new(repo: PermissionRepository) -> Self {
        Self { repo }
    }

    pub async fn create_default_permissions(&self, company_id: &str) {
        // TODO: Setup default permissions
    }

    pub async fn create(
        &self,
        permission: PermissionEntity,
    ) -> Result<Option<PermissionEntity>, anyhow::Error> {
        // TODO: Validate the data
        self.repo.create(permission).await
    }

    pub async fn update(
        &self,
        mut permission: PermissionEntity,
    ) -> Result<Option<PermissionEntity>, anyhow::Error> {
        let the_id = permission.id.clone();

        if let Some(id) = &the_id {
            // TODO: Validate the data

            // Name should never be changed
            permission.name = None;

            self.repo.update(permission, id).await
        } else {
            Err(anyhow!("Permission does not have an ID"))
        }
    }
}

#[busybody::async_trait]
impl busybody::Injectable for PermissionService {
    async fn inject(ci: &busybody::ServiceContainer) -> Self {
        let repo: PermissionRepository = ci.provide().await;

        Self::new(repo)
    }
}
