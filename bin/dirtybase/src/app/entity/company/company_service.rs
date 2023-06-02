use dirtybase_db::entity::user::UserEntity;

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
        &mut self,
        company: CompanyEntity,
        user: UserEntity,
    ) -> Result<CompanyEntity, anyhow::Error> {
        unimplemented!()
    }

    pub async fn update(
        &mut self,
        company: CompanyEntity,
        id: &str,
    ) -> Result<CompanyEntity, anyhow::Error> {
        unimplemented!()
    }
}
