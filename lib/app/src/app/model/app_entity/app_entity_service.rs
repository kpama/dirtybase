use super::{AppEntity, AppRepository};
use anyhow::anyhow;
use dirtybase_contract::db::entity::user::UserEntity;
use dirtybase_db::base::helper::generate_ulid;

pub struct AppEntityService {
    app_repo: AppRepository,
}

impl AppEntityService {
    pub fn new(app_repo: AppRepository) -> Self {
        Self { app_repo }
    }

    pub fn app_repo(&self) -> &AppRepository {
        &self.app_repo
    }

    pub fn app_repo_mut(&mut self) -> &mut AppRepository {
        &mut self.app_repo
    }

    pub fn new_app(&self) -> AppEntity {
        AppEntity::new()
    }

    pub async fn create(
        &self,
        mut app: AppEntity,
        blame: UserEntity,
    ) -> Result<Option<AppEntity>, anyhow::Error> {
        // TODO: Validation....
        if app.core_company_id.is_none() {
            return Err(anyhow!("An application requires a company"));
        }

        if app.name.is_none() {
            return Err(anyhow!("Application must have a name"));
        }

        if blame.id.is_none() {
            return Err(anyhow!(
                "Some has to be blamed for creating this application"
            ));
        }

        if app.id.is_none() {
            app.id = Some(generate_ulid());
        }

        app.creator_id = Some(blame.id.unwrap());

        self.app_repo().create(app).await
    }

    pub async fn update(
        &self,
        mut app: AppEntity,
        id: &str,
        blame: UserEntity,
    ) -> Result<Option<AppEntity>, anyhow::Error> {
        // TODO: Validation
        if app.core_company_id.is_none() {
            return Err(anyhow!("An application requires a company"));
        }

        if app.name.is_none() {
            return Err(anyhow!("Application must have a name"));
        }

        if blame.id.is_none() {
            return Err(anyhow!(
                "Some has to be blamed for creating this application"
            ));
        }

        app.editor_id = Some(blame.id.unwrap());
        self.app_repo().update(id, app).await
    }
}

#[busybody::async_trait]
impl busybody::Injectable for AppEntityService {
    async fn inject(ci: &busybody::ServiceContainer) -> Self {
        let app_repo = ci.provide::<AppRepository>().await;

        Self::new(app_repo)
    }
}
