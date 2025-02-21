use anyhow::anyhow;
use orsomafo::Dispatchable;
use sha2::{Digest, Sha256};

use dirtybase_contract::db::{base::helper::generate_ulid, event::UserCreatedEvent};

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

    pub async fn get_user_by_username(
        &self,
        username: &str,
        without_trash: bool,
    ) -> Result<Option<UserEntity>, anyhow::Error> {
        self.user_repo
            .find_by_username(username, without_trash)
            .await
    }

    /// Returns or create the default system administrator
    /// A result is returned where Result<(true => user was created, the user record), Error>`
    pub async fn create_admin_user(
        &mut self,
        username: &str,
        email: &str,
        raw_password: &str,
    ) -> Result<Option<(bool, UserEntity)>, anyhow::Error> {
        if let Ok(Some(user)) = self
            .user_repo
            .find_by_username_and_email(username, email, true)
            .await
        {
            Ok(Some((false, user)))
        } else {
            let mut user = UserEntity::new();
            user.email = Some(email.into());
            user.username = Some(username.into());
            user.password = Some(raw_password.into());
            user.reset_password = Some(true);
            user.status = Some(super::UserStatus::Active);
            user.is_sys_admin = true;

            let mut hash = Sha256::new();
            hash.update(generate_ulid().as_bytes());
            user.salt = Some(format!("{:x}", hash.finalize()));

            match self.create(user).await {
                Ok(result) => match result {
                    Some(user) => Ok(Some((true, user))),
                    None => Err(anyhow!("could not create user")),
                },
                Err(e) => Err(e),
            }
        }
    }

    pub async fn reset_password(
        &mut self,
        password: &str,
        id: &str,
    ) -> Result<Option<UserEntity>, anyhow::Error> {
        // TODO: validate password
        let user = UserEntity {
            password: Some(hash_password(password)),
            ..UserEntity::default()
        };

        self.update(user, id).await
    }

    pub async fn create(
        &mut self,
        mut user: UserEntity,
    ) -> Result<Option<UserEntity>, anyhow::Error> {
        user.password = Some(hash_password(
            &user.password.unwrap_or("changeme!!".to_owned()),
        ));

        let result = self.user_repo.create(user).await;

        if let Ok(Some(user)) = &result {
            let event = UserCreatedEvent::new(user.id.clone());
            event.dispatch_event();
        } else {
            log::error!(
                "could not create user: {:#?}",
                result.as_ref().err().unwrap()
            );
        }

        result
    }

    pub async fn update(
        &mut self,
        user: UserEntity,
        id: &str,
    ) -> Result<Option<UserEntity>, anyhow::Error> {
        self.user_repo.update(id, user).await
    }
}
