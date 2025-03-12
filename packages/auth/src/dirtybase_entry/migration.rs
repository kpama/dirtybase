mod mig_1740151519_create_auth_user_table;

/**
 * The following function is automatically modified
 * do not manually edit it
 */
pub(crate) fn setup() -> Option<dirtybase_contract::ExtensionMigrations> {
    dirtybase_contract::register_migration![
        mig_1740151519_create_auth_user_table::Mig1740151519CreateAuthUserTable,
        //
    ]
}
