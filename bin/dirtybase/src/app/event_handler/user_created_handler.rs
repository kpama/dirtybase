use crate::app::model::dirtybase_user::DirtybaseUserService;
use busybody::helpers::provide;
use dirtybase_db::db::event::UserCreatedEvent;

#[derive(Default)]
pub(crate) struct UserCreatedHandler;

#[orsomafo::async_trait]
impl orsomafo::EventHandler for UserCreatedHandler {
    async fn handle(&self, dispatched: &orsomafo::DispatchedEvent) {
        let event: UserCreatedEvent = dispatched.the_event().unwrap();
        let dirtybase_user_service = provide::<DirtybaseUserService>().await;
        let mut user = dirtybase_user_service.new_user();

        // Fields
        user.core_user_id = Some(event.id());
        user.generate_salt();

        let result = dirtybase_user_service.create(user).await;
        if result.is_err() {
            log::error!(
                "could not create dirtybase user from core user: {:#?}",
                result.err().unwrap()
            );
        }
    }
}
