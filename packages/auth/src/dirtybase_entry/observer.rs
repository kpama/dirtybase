mod cel;
mod guard;

pub(crate) async fn register_observers() {
    guard::register_guard_observers().await;
    cel::register_observers().await;
}
