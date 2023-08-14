use super::pipeline;

pub async fn setup_default_entities() {
    pipeline::setup_system_wide_admin_pipeline::execute().await;
}
