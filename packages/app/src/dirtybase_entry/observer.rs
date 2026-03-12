mod cel;

pub(crate) async fn register_observers() {
    cel::register_observers().await;
}
