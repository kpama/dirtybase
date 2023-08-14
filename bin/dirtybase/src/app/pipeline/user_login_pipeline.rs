use crate::app::entity::dirtybase_user::{
    dirtybase_user_helpers::authentication_error_status::AuthenticationErrorStatus,
    dtos::in_user_login_payload_dto::UserLoginPayload, DirtybaseUserService,
};
use fama::PipeContent;

type AuthResult = Result<UserLoginPayload, AuthenticationErrorStatus>;

pub(crate) async fn execute(payload: UserLoginPayload) -> AuthResult {
    fama::Pipeline::pass(payload)
        // 0. Find cached user's data
        // 1. If the user is blocked
        //   1.1 End the process
        // 2. Find the user actual data
        //   2.1. Try logging in the user
        //   2.2. If login is successful
        //     2.2.1 Reset user attempt count
        //     2.2.2 Log last login timestamp
        //     2.2.3 Dispatch login failed event
        // 2.3 If login failed
        //     2.3.1 increment login attempt count
        //     2.3.2 If the user has exceeded their attempt
        //         2.3.2.1 Block the user from further login attempt
        //         2.3.2.2 Notify app Admin if the user is logging in to an app directly
        // 2.4 Cache the user data
        // 2.5 Return result
        .deliver_as::<AuthResult>()
}

pub async fn pass_through_user_login_pipeline(payload: UserLoginPayload) {
    let x: AuthResult = fama::Pipeline::pass(payload)
        .through_fn(find_user)
        .await
        .through_fn(
            |pipe: PipeContent, payload: UserLoginPayload, service: DirtybaseUserService| async move {
                pipe.container().set_type::<AuthResult>(Ok(payload));
                None
            },
        )
        .await
        .deliver_as();

    dbg!(x);
}

async fn find_user(
    pipe: PipeContent,
    payload: UserLoginPayload,
    service: DirtybaseUserService,
) -> Option<PipeContent> {
    let username = payload.username.unwrap_or_default();
    let email = payload.email.unwrap_or_default();

    let result = service
        .dirtybase_user_repo()
        .find_by_username_or_email(&username, &email, true)
        .await
        .map_err(|e| {
            log::info!("fetching user error: {:?}", e);
            AuthenticationErrorStatus::UserNotFound
        });

    dbg!(&result);

    pipe.store(result);

    Some(pipe)
}
