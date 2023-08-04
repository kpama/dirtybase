use orsomafo::Dispatchable;

use crate::{base::helper::generate_ulid, event::UserCreatedEvent};

use super::{hash_password, user_repository::UserRepository, UserEntity};

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

    /// Returns or create the default system administrator
    /// A result is returned where Result<(true => user was created, the user record), Error>`
    pub async fn create_admin_user(
        &mut self,
        username: &str,
        email: &str,
        raw_password: &str,
    ) -> Result<(bool, UserEntity), anyhow::Error> {
        if let Ok(user) = self
            .user_repo
            .find_by_username_and_email(username, email, true)
            .await
        {
            Ok((false, user))
        } else {
            let mut user = UserEntity::new();
            user.email = Some(email.into());
            user.username = Some(username.into());
            user.password = Some(raw_password.into());
            user.reset_password = Some(true);
            user.status = Some(super::UserStatus::Active);

            match self.create(user).await {
                Ok(user) => Ok((true, user)),
                Err(e) => Err(e),
            }
        }
    }

    pub async fn reset_password(
        &mut self,
        password: &str,
        id: &str,
    ) -> Result<UserEntity, anyhow::Error> {
        let mut user = UserEntity::default();
        // TODO: validate password
        user.password = Some(hash_password(password));

        self.update(user, id).await
    }

    pub async fn create(&mut self, mut user: UserEntity) -> Result<UserEntity, anyhow::Error> {
        user.password = Some(hash_password(
            &user.password.unwrap_or("changeme!!".to_owned()),
        ));

        if user.id.is_none() {
            user.id = Some(generate_ulid());
        }

        // TODO: dispatch user created event
        let event = UserCreatedEvent::new(user.id.as_ref().unwrap());
        event.dispatch_event();

        self.user_repo.create(user).await
    }

    pub async fn update(
        &mut self,
        user: UserEntity,
        id: &str,
    ) -> Result<UserEntity, anyhow::Error> {
        self.user_repo.update(id, user).await
    }
}
