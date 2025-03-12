mod mig_0000000000_create_user_table;

/**
 * The following function is automatically modified
 * do not manually edit it
 */
pub(crate) fn setup() -> Option<dirtybase_contract::ExtensionMigrations> {
    dirtybase_contract::register_migration![
        mig_0000000000_create_user_table::Mig0000000000CreateUserTable,
        //
    ]
}
