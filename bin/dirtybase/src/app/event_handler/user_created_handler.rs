use busybody::helpers::provide;
use dirtybase_db::event::UserCreatedEvent;

use crate::app::entity::dirtybase_user::DirtybaseUserService;

#[derive(Default)]
pub(crate) struct UserCreatedHandler;

#[orsomafo::async_trait]
impl orsomafo::EventHandler for UserCreatedHandler {
    async fn handle(&self, dispatched: &orsomafo::DispatchedEvent) {
        let event: UserCreatedEvent = dispatched.the_event().unwrap();
        let mut dirtybase_user_sevice = provide::<DirtybaseUserService>().await;
        let mut user = dirtybase_user_sevice.new_user();

        // Fields
        user.core_user_id = Some(event.id());

        let result = dirtybase_user_sevice.create(user).await;
        if result.is_err() {
            log::error!(
                "could not create dirtybase user from core user: {:#?}",
                result.err().unwrap()
            );
        }
    }
}
