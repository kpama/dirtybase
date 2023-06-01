use super::{user_repository::UserRepository, UserEntity};

pub struct UserService {
    repo: UserRepository,
}

impl UserService {
    pub fn new(repo: UserRepository) -> Self {
        Self { repo }
    }

    pub async fn create_admin_user(
        &self,
        name: &str,
        email: &str,
        raw_password: &str,
    ) -> Result<UserEntity, anyhow::Error> {
        let user = UserEntity::default();

        Ok(user)
    }
}
