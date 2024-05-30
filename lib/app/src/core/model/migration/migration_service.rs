use super::MigrationRepository;

pub struct MigrationService {
    repo: MigrationRepository,
}

impl MigrationService {
    pub fn new(repo: MigrationRepository) -> Self {
        Self { repo }
    }

    pub fn repo(&self) -> &MigrationRepository {
        &self.repo
    }
}

#[busybody::async_trait]
impl busybody::Injectable for MigrationService {
    async fn inject(c: &busybody::ServiceContainer) -> Self {
        let (repo,) = c.inject_all::<(MigrationRepository,)>().await;

        Self::new(repo)
    }
}
