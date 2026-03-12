pub mod mig_1740151519_create_auth_user_table;
pub mod mig_1762480990_create_permission_tables;

/**
 * The following function is automatically modified
 * do not manually edit it
 */
pub(crate) fn setup() -> Option<dirtybase_contract::ExtensionMigrations> {
    dirtybase_contract::register_migration![
        #[cfg(feature = "permission")]
        mig_1762480990_create_permission_tables::Mig1762480990CreatePermissionTables,
        mig_1740151519_create_auth_user_table::Mig1740151519CreateAuthUserTable,
        //
    ]
}
