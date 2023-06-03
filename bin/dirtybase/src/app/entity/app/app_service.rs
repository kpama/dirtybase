use anyhow::anyhow;
use dirtybase_db::{base::helper::generate_ulid, entity::user::UserEntity};

use super::{AppEntity, AppRepository};

pub struct AppService {
    app_repo: AppRepository,
}

impl AppService {
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
        &mut self,
        mut app: AppEntity,
        blame: UserEntity,
    ) -> Result<AppEntity, anyhow::Error> {
        // TODO: Validation....
        if app.company_id.is_none() {
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

        return self.app_repo_mut().create(app).await;
    }

    pub async fn update(
        &mut self,
        mut app: AppEntity,
        id: &str,
        blame: UserEntity,
    ) -> Result<AppEntity, anyhow::Error> {
        // TODO: Validation
        if app.company_id.is_none() {
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
        self.app_repo_mut().update(id, app).await
    }
}
