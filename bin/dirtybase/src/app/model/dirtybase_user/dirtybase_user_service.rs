use crate::app::{
    token_claim::{ClaimBuilder, JWTClaim},
    DirtyBaseApp,
};

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
use busybody::helpers::provide;
use dirtybase_cache::CacheManager;
use dirtybase_db::entity::user::{
    verify_password, UserEntity, UserRepository, UserService, UserStatus,
};

pub struct DirtybaseUserService {
    repo: DirtybaseUserRepository,
    cache: CacheManager,
    user_service: UserService,
}

impl DirtybaseUserService {
    pub fn new(
        repo: DirtybaseUserRepository,
        cache: CacheManager,
        user_service: UserService,
    ) -> Self {
        Self {
            repo,
            cache,
            user_service,
        }
    }

    pub(crate) fn cache(&self) -> &CacheManager {
        &self.cache
    }
    pub fn dirtybase_user_repo(&self) -> &DirtybaseUserRepository {
        &self.repo
    }

    pub fn dirtybase_user_repo_mut(&mut self) -> &mut DirtybaseUserRepository {
        &mut self.repo
    }

    pub fn user_service(&self) -> &UserService {
        &self.user_service
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
            Err(anyhow::anyhow!("Error generating user's application token"))
        }
    }

    pub async fn log_user_in(
        &self,
        mut user: DirtybaseUserEntity,
        password: &str,
    ) -> Result<LoggedInUser, AuthenticationErrorStatus> {
        if verify_password(password, &user.user.password.as_ref().unwrap()) {
            let mut out_dto: LoggedInUser = user.clone().into();

            // JWT token
            if let Some(token) = ClaimBuilder::new(&user)
                .set_allow(JWTClaim::CanSwitchAp)
                .generate()
                .await
            {
                out_dto.token = token;
            }

            user.reflect_login_success();
            _ = self.repo.update(user).await;

            Ok(out_dto)
        } else {
            user.reflect_login_failure();
            _ = self.repo.update(user.clone()).await;
            Err(AuthenticationErrorStatus::AuthenticationFailed)
        }
    }

    pub async fn reset_login_attempts(&self, core_user_id: &str) -> Result<bool, anyhow::Error> {
        match self.repo.find_by_core_user_id(core_user_id).await {
            Ok(mut user) => {
                user.reset_login_attempts();
                _ = self.repo.update(user).await;
                Ok(true)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn can_login(&self, user: &LoggedInUser) -> Result<bool, AuthenticationErrorStatus> {
        // check

        // 1. User attempts
        // TODO: Make the hard coded `10` configurable
        if user.login_attempt >= 10 {
            log::error!("too many attempts");
            return Err(AuthenticationErrorStatus::TooManyAttempts);
        }
        // 2. Account is suspended
        if user.user.status == UserStatus::Suspended {
            return Err(AuthenticationErrorStatus::AccountSuspended);
        }

        // 3. Account is inactive
        if user.user.status == UserStatus::Inactive {
            return Err(AuthenticationErrorStatus::AccountInactive);
        }

        Ok(true)
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
            Ok(user) => self.log_user_in(user, &password).await,
            Err(_) => return Err(AuthenticationErrorStatus::UserNotFound),
        }
    }
}

#[busybody::async_trait]
impl busybody::Injectable for DirtybaseUserService {
    async fn inject(ci: &busybody::ServiceContainer) -> Self {
        let repo = ci.provide::<DirtybaseUserRepository>().await;
        let app = ci.get::<DirtyBaseApp>().unwrap();
        let cache = provide::<CacheManager>().await.tags(&["dtb_user"]).await;

        let user_service = UserService::new(UserRepository::new(app.schema_manger()));

        Self::new(repo, cache, user_service)
    }
}
