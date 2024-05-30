use fama::PipelineBuilderTrait;

use super::pipeline::setup_system_wide_admin_pipeline::NewSysAdminData;

pub async fn setup_default_entities() {
    if NewSysAdminData::default().pipeline().await.confirm() {
        log::info!("Default administrator user created");
    } else {
        panic!("Could not create default administrator")
    }
}
