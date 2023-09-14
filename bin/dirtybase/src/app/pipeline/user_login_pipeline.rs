use crate::app::model::dirtybase_user::DirtybaseUserEntity;
use crate::app::model::dirtybase_user::{
    dirtybase_user_helpers::authentication_error_status::AuthenticationErrorStatus,
    dtos::{in_user_login_payload_dto::UserLoginPayload, out_logged_in_user_dto::LoggedInUser},
    DirtybaseUserService,
};
use fama::PipeContent;

type AuthResult = Result<LoggedInUser, AuthenticationErrorStatus>;

const LOG_TARGET: &str = "pipeline:user_login";

impl From<AuthenticationErrorStatus> for AuthResult {
    fn from(value: AuthenticationErrorStatus) -> Self {
        Self::Err(value)
    }
}

pub(crate) async fn execute(payload: UserLoginPayload) -> AuthResult {
    fama::Pipeline::pass(payload)
        // Validate data
        .through_fn(validate_payload)
        .await
        // 0. Find cached user's data
        .through_fn(find_cached_user_data)
        .await
        // 1. If the user is blocked
        //   1.1 End the process
        .through_fn(check_if_user_is_blocked)
        .await
        // 2. Find the user actual data
        .through_fn(find_user)
        .await
        //   2.1. Try logging in the user
        //   2.2. If login is successful
        //     2.2.1 Reset user attempt count
        //     2.2.2 Log last login timestamp
        //     2.2.3 Dispatch login succeeded  event
        // 2.3 If login failed
        //     2.3.1 increment login attempt count
        //     2.3.2 If the user has exceeded their attempt
        //         2.3.2.1 Block the user from further login attempt
        //         2.3.2.2 Notify app Admin if the user is logging in to an app directly
        //         2.3.2.3 Dispatch login failed event
        // 2.4 Cache the user data
        .through_fn(try_logging_user_in)
        .await
        // 2.5 Return result
        .deliver_as::<AuthResult>()
}

async fn validate_payload(mut pipe: PipeContent) -> Option<PipeContent> {
    let payload: UserLoginPayload = pipe.container().get_type().unwrap();
    let result = payload.validate();
    if result.is_err() {
        pipe.store::<AuthResult>(result.err().unwrap().into());
        pipe.stop_the_flow();
    }

    Some(pipe)
}

async fn find_cached_user_data(
    pipe: PipeContent,
    payload: UserLoginPayload,
    service: DirtybaseUserService,
) -> Option<PipeContent> {
    let cache_id = payload.cache_id();
    log::debug!(
        target: LOG_TARGET,
        " 0. Find cached user's data"
    );

    if let Some(user) = service.cache().get::<LoggedInUser>(&cache_id).await {
        // TODO: check the cached version
        dbg!("cached user data: {:#?}", user);
    }

    Some(pipe)
}

async fn check_if_user_is_blocked(pipe: PipeContent) -> Option<PipeContent> {
    log::debug!(
        target: LOG_TARGET,
        "1. If the user is blocked"
    );

    Some(pipe)
}

async fn find_user(
    mut pipe: PipeContent,
    payload: UserLoginPayload,
    service: DirtybaseUserService,
) -> Option<PipeContent> {
    let username = payload.username.unwrap_or_default();
    let email = payload.email.unwrap_or_default();

    log::debug!(
        target: LOG_TARGET,
        "2. Find the user actual data"
    );

    let result = service
        .dirtybase_user_repo()
        .find_by_username_or_email(&username, &email, true)
        .await
        .map_err(|_| AuthenticationErrorStatus::UserNotFound);

    if result.is_ok() {
        pipe.store(result.unwrap());
    } else {
        pipe.store(AuthResult::Err(result.err().unwrap()));
        pipe.stop_the_flow();
    }

    Some(pipe)
}

async fn try_logging_user_in(
    mut pipe: PipeContent,
    service: DirtybaseUserService,
    payload: UserLoginPayload,
) -> Option<PipeContent> {
    log::debug!(
        target: LOG_TARGET,
        "2. Find the user actual data"
    );
    // let service: DirtybaseUserService = provide().await;
    // let payload: UserLoginPayload = pipe.container().get_type().unwrap();
    let cache_id = payload.cache_id();

    if let Some(user) = pipe.container().get_type::<DirtybaseUserEntity>() {
        let result = service.log_user_in(user.clone(), &payload.password).await;
        if result.is_err() {
            // TODO: !!!
            // 2.3 If login failed
            //     2.3.1 increment login attempt count
            //     2.3.2 If the user has exceeded their attempt
            //         2.3.2.1 Block the user from further login attempt
            //         2.3.2.2 Notify app Admin if the user is logging in to an app directly
            //         2.3.2.3 Dispatch login failed event

            pipe.store(AuthResult::Err(result.err().unwrap()));
        } else {
            service
                .cache()
                .put(&cache_id, result.as_ref().unwrap(), None)
                .await;
            pipe.store(AuthResult::Ok(result.unwrap()));
        }
    } else {
        pipe.store::<AuthResult>(AuthenticationErrorStatus::AuthenticationFailed.into());
        pipe.stop_the_flow();
    }

    Some(pipe)
}
struct CachedLoggedUser(LoggedInUser);
