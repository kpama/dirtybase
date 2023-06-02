use super::{
    hash_password, user_entity::UserUpdateEntity, user_repository::UserRepository, UserEntity,
};

pub struct UserService {
    user_repo: UserRepository,
}

impl UserService {
    pub fn new(user_repo: UserRepository) -> Self {
        Self { user_repo }
    }

    pub fn user_repo(&self) -> &UserRepository {
        &self.user_repo
    }

    pub fn user_repo_mut(&mut self) -> &mut UserRepository {
        &mut self.user_repo
    }

    pub async fn create_admin_user(
        &mut self,
        username: &str,
        email: &str,
        raw_password: &str,
    ) -> Result<UserEntity, anyhow::Error> {
        if let Ok(user) = self
            .user_repo
            .find_one_by_username_and_email(username, email)
            .await
        {
            dbg!(&user);
            Ok(user)
        } else {
            let mut user = UserEntity::default();
            user.set_email(&email)
                .set_username(&username)
                .set_password(&hash_password(&raw_password))
                .set_reset_password(true);

            self.save(user, None).await
        }
    }

    pub async fn save(
        &mut self,
        user: UserEntity,
        id: Option<&str>,
    ) -> Result<UserEntity, anyhow::Error> {
        let record: UserUpdateEntity = user.into();
        if let Some(id) = id {
            self.user_repo.update(id, record).await
        } else {
            self.user_repo.create(record).await
        }
    }
}
