use crate::app::token_claim::{ClaimBuilder, JWTClaim};

use super::{
    dirtybase_user_entity::DirtybaseUserEntity,
    dirtybase_user_helpers::authentication_error_status::AuthenticationErrorStatus,
    dirtybase_user_repository::DirtybaseUserRepository,
    dtos::{
        in_switch_app_dto::SwitchAppDto, in_user_login_payload_dto::UserLoginPayload,
        out_logged_in_user_dto::LoggedInUser, out_switch_app_result_dto::SwitchAppResultDto,
    },
};
use anyhow::anyhow;
use dirtybase_db::entity::user::{verify_password, UserEntity};

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
        &self,
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

    pub async fn generate_app_token(
        &self,
        core_user_id: &str,
        payload: SwitchAppDto,
    ) -> Result<SwitchAppResultDto, anyhow::Error> {
        // TODO: Validation
        if let Ok(user) = self.repo.find_by_core_user_id(core_user_id).await {
            Ok(ClaimBuilder::new(&user)
                .set_app(&payload.app_id)
                .set_role(&payload.role_id)
                .generate()
                .await
                .unwrap()
                .into())
        } else {
            Err(anyhow::anyhow!("Error generating user's applicaiton token"))
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
                if verify_password(&password, &user.user.password.as_ref().unwrap()) {
                    let mut dto: LoggedInUser = user.clone().into();

                    // JWT token
                    dto.token = ClaimBuilder::new(&user)
                        .set_allow(JWTClaim::CanSwitchAp)
                        .generate()
                        .await
                        .unwrap();

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
