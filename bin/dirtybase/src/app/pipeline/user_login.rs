use crate::app::entity::dirtybase_user::{
    dirtybase_user_helpers::authentication_error_status::AuthenticationErrorStatus,
    dtos::in_user_login_payload_dto::UserLoginPayload, DirtybaseUserService,
};
use fama::PipeContent;

type AuthResult = Result<UserLoginPayload, AuthenticationErrorStatus>;

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
