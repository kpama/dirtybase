mod mig_1698982353_create_main_tables;
mod mig_1698982370_setup_system_admin_user;

/**
 * The following function is automatically modified
 * do not manually edit it
 */
pub(crate) fn setup() -> Vec<Box<dyn dirtybase_contract::db::migration::Migration>> {
    vec![
        Box::new(mig_1698982353_create_main_tables::Mig1698982353createmaintables),
        Box::new(mig_1698982370_setup_system_admin_user::Mig1698982370setupsystemadminuser),
        // dty_inject
    ]
}
