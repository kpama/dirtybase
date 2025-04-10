mod mig_1744202277_create_session_table;

/**
 * The following function is automatically modified
 * do not manually edit it
 */
pub(crate) fn setup() -> Option<dirtybase_contract::ExtensionMigrations> {
    dirtybase_contract::register_migration![
        mig_1744202277_create_session_table::Mig1744202277CreateSessionTable,
        //
    ]
}
