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
