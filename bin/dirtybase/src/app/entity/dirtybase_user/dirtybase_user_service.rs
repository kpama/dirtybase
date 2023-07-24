use super::{
    dirtybase_user_entity::DirtybaseUserEntity,
    dirtybase_user_helpers::{
        authentication_error_status::AuthenticationErrorStatus, jwt_manager::JWTManager,
    },
    dirtybase_user_repository::DirtybaseUserRepository,
    in_dtos::UserLoginPayload,
    out_dtos::LoggedInUser,
};
use anyhow::anyhow;
use busybody::helpers::provide;
use dirtybase_db::entity::user::{verify_password, UserEntity};
use std::collections::HashMap;

pub struct DirtybaseUserService {
    repo: DirtybaseUserRepository,
}

impl DirtybaseUserService {
    pub fn new(repo: DirtybaseUserRepository) -> Self {
        Self { repo }
    }

    pub fn dirtybase_user_repo(&self) -> &DirtybaseUserRepository {
        &self.repo
    }

    pub fn dirtybase_user_repo_mut(&mut self) -> &mut DirtybaseUserRepository {
        &mut self.repo
    }

    pub fn new_user(&self) -> DirtybaseUserEntity {
        DirtybaseUserEntity::default()
    }

    pub async fn create(
        &mut self,
        entity: DirtybaseUserEntity,
    ) -> Result<DirtybaseUserEntity, anyhow::Error> {
        if entity.core_user_id.is_some() {
            return self.repo.create(entity).await;
        }

        Err(anyhow!("Core user ID is required"))
    }

    pub async fn authenticate_password(&self, password: &str, user: &UserEntity) -> bool {
        if password.trim().is_empty() {
            false
        } else {
            verify_password(password, user.password.as_ref().unwrap())
        }
    }

    pub async fn login(
        &self,
        payload: UserLoginPayload,
    ) -> Result<LoggedInUser, AuthenticationErrorStatus> {
        let username = payload.username.unwrap_or_default();
        let email = payload.email.unwrap_or_default();
        let password = payload.password;

        if password.is_empty() {
            return Err(AuthenticationErrorStatus::AuthenticationFailed);
        }

        match self
            .dirtybase_user_repo()
            .find_by_username_or_email(&username, &email, true)
            .await
        {
            Ok(user) => {
                if verify_password(&password, &user.password.as_ref().unwrap()) {
                    let jwt_manager = provide::<JWTManager>().await;
                    let mut dto = LoggedInUser::from(user);

                    // TODO: This shouldn't be here....
                    let mut claim = HashMap::new();
                    claim.insert("foo".into(), "bar".into());

                    dto.token = jwt_manager.sign_to_jwt(claim);

                    Ok(dto)
                } else {
                    Err(AuthenticationErrorStatus::AuthenticationFailed)
                }
            }
            Err(_) => return Err(AuthenticationErrorStatus::UserNotFound),
        }
    }
}

#[busybody::async_trait]
impl busybody::Injectable for DirtybaseUserService {
    async fn inject(ci: &busybody::ServiceContainer) -> Self {
        let repo = ci.provide::<DirtybaseUserRepository>().await;
        Self::new(repo)
    }
}
