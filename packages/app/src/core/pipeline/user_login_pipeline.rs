use crate::core::model::dirtybase_user::dirtybase_user_cache::can_login_cache::CanLoginCachedData;
use crate::core::model::dirtybase_user::DirtybaseUserEntity;
use crate::core::model::dirtybase_user::{
    dirtybase_user_helpers::authentication_error_status::AuthenticationErrorStatus,
    dtos::{in_user_login_payload_dto::UserLoginPayload, out_logged_in_user_dto::LoggedInUser},
    DirtybaseUserService,
};
use fama::{PipeContent, PipelineBuilderTrait};

type AuthResult = Result<LoggedInUser, AuthenticationErrorStatus>;

const LOG_TARGET: &str = "pipeline:user_login";

impl From<AuthenticationErrorStatus> for AuthResult {
    fn from(value: AuthenticationErrorStatus) -> Self {
        Self::Err(value)
    }
}

pub(crate) async fn execute(payload: UserLoginPayload) -> AuthResult {
    payload
        .pipeline()
        .await
        .try_deliver_as::<AuthResult>()
        .await
        .unwrap_or_else(|| AuthenticationErrorStatus::AuthenticationFailed.into())
}

pub(crate) async fn register_pipes(
    builder: fama::PipelineBuilder<UserLoginPayload>,
) -> fama::PipelineBuilder<UserLoginPayload> {
    builder
        .register(|pipeline| {
            Box::pin(async {
                pipeline
                    .ok_fn(validate_payload)
                    .await
                    .store_fn(inject_user_email)
                    .await
                    .store_fn(find_can_user_login_cache_data)
                    .await
                    .next_fn(check_if_user_can_login)
                    .await
                    .some_fn(find_user)
                    .await
                    .ok_fn(try_logging_user_in)
                    .await
            })
        })
        .await;
    builder
}

async fn validate_payload(
    payload: UserLoginPayload,
) -> Result<UserLoginPayload, AuthenticationErrorStatus> {
    let result = payload.validate();
    if result.is_err() {
        return Err(result.err().unwrap());
    }

    Ok(payload)
}

async fn inject_user_email(
    mut payload: UserLoginPayload,
    service: DirtybaseUserService,
) -> UserLoginPayload {
    if payload.email.is_none() {
        if let Ok(Some(user)) = service
            .user_service()
            .get_user_by_username(payload.username.as_ref().unwrap(), true)
            .await
        {
            println!("user email -----> : {:#?}", &user.email);
            payload.email = user.email;
        }
    }

    payload
}

async fn find_can_user_login_cache_data(
    payload: UserLoginPayload,
    service: DirtybaseUserService,
) -> Option<CanLoginCachedData> {
    if let Some(email) = payload.email.as_ref() {
        return CanLoginCachedData::fetch(service.cache(), email).await;
    }
    None
}

async fn check_if_user_can_login(pipe: PipeContent) -> bool {
    let result = pipe
        .container()
        .get_type::<Option<CanLoginCachedData>>()
        .await;

    if let Some(Some(data)) = result {
        if !data.allow {
            pipe.store::<AuthResult>(AuthResult::Err(data.error));
            return false;
        }
    }

    true
}

async fn find_user(
    pipe: PipeContent,
    payload: UserLoginPayload,
    service: DirtybaseUserService,
) -> Option<DirtybaseUserEntity> {
    let username = payload.username.unwrap_or_default();
    let email = payload.email.unwrap_or_default();

    log::debug!(target: LOG_TARGET, "2. Find the user actual data" );

    match service
        .dirtybase_user_repo()
        .find_by_username_or_email(&username, &email, true)
        .await
    {
        Ok(dirty_user) => Some(dirty_user),
        Err(_) => {
            pipe.store::<AuthResult>(AuthResult::Err(AuthenticationErrorStatus::UserNotFound));
            None
        }
    }
}

async fn try_logging_user_in(
    pipe: PipeContent,
    service: DirtybaseUserService,
    payload: UserLoginPayload,
) -> AuthResult {
    log::debug!(
        target: LOG_TARGET,
        "2. Find the user actual data"
    );
    let user = pipe
        .container()
        .get_type::<Option<DirtybaseUserEntity>>()
        .await
        .unwrap()
        .unwrap();
    let result = service.log_user_in(user.clone(), &payload.password).await;

    // 2.3 If login failed
    //     2.3.1 increment login attempt count
    //     2.3.2 If the user has exceeded their attempt
    //         2.3.2.1 Block the user from further login attempt
    //         2.3.2.2 Notify app Admin if the user is logging in to an app directly
    //         2.3.2.3 Dispatch login failed event
    // 2.4 Cache the user data

    if result.is_err() {
        let error = result.err().unwrap();

        match service.can_login(&user.into()).await {
            Ok(_) => AuthResult::Err(error),
            Err(e) => {
                if let Some(email) = payload.email.as_ref() {
                    CanLoginCachedData::store(service.cache(), email, false, e.clone()).await;
                }
                Err(e)
            }
        }
    } else {
        let logged_in_user = result.unwrap();
        match service.can_login(&logged_in_user).await {
            Ok(_) => AuthResult::Ok(logged_in_user),
            Err(e) => {
                if let Some(email) = payload.email.as_ref() {
                    CanLoginCachedData::store(service.cache(), email, false, e.clone()).await;
                }
                Err(e)
            }
        }
    }
}
