use anyhow::anyhow;
use dirtybase_user::entity::user::UserEntity;

use super::{CompanyEntity, CompanyRepository};

pub struct CompanyService {
    company_repo: CompanyRepository,
}

impl CompanyService {
    pub fn new(company_repo: CompanyRepository) -> Self {
        Self { company_repo }
    }

    pub fn company_repo(&self) -> &CompanyRepository {
        &self.company_repo
    }

    pub fn company_repo_mut(&mut self) -> &mut CompanyRepository {
        &mut self.company_repo
    }

    pub async fn create(
        &self,
        mut company: CompanyEntity,
        company_user: UserEntity,
        blame: UserEntity,
    ) -> Result<Option<CompanyEntity>, anyhow::Error> {
        // TODO: validate the record
        if company.name.is_empty() {
            return Err(anyhow!("Name field is required")); // TODO: Insert this in map and return all the errors at once?
        }

        if company_user.id.is_empty() {
            return Err(anyhow!("A company must have a super administrator"));
        }

        if blame.id.is_empty() {
            return Err(anyhow!("Company entity requires a user to blame"));
        }

        // prep
        if company.id.is_empty() {
            company.id = Default::default();
        }
        company.core_user_id = company_user.id;
        company.creator_id = blame.id;

        self.company_repo.create(company).await
    }

    pub async fn update(
        &self,
        mut company: CompanyEntity,
        id: &str,
        blame: UserEntity,
    ) -> Result<Option<CompanyEntity>, anyhow::Error> {
        // TODO: Validation ....
        if blame.id.is_empty() {
            return Err(anyhow!("Company entity requires a user to blame"));
        }

        company.editor_id = Some(blame.id);
        self.company_repo.update(id, company).await
    }
}

#[busybody::async_trait]
impl busybody::Injectable for CompanyService {
    async fn inject(c: &busybody::ServiceContainer) -> Self {
        let (company_repo,) = c.inject_all::<(CompanyRepository,)>().await;
        Self::new(company_repo)
    }
}
