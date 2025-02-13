pub(crate) mod mig_1698982353_create_main_tables;

/**
 * The following function is automatically modified
 * do not manually edit it
 */
pub(crate) fn setup() -> Vec<Box<dyn dirtybase_contract::db::migration::Migration>> {
    vec![Box::new(
        mig_1698982353_create_main_tables::Mig1698982353CreateMainTables,
    )]
}
