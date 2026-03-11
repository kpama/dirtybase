mod guard;

pub async fn register_observers() {
    guard::register_guard_observers().await;
}
