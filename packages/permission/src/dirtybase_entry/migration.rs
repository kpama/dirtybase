mod mig_1762480990_create_permission_tables;
/**
 * The following function is automatically modified
 * do not manually edit it
 */
pub(crate) fn setup() -> Option<dirtybase_contract::ExtensionMigrations> {
    dirtybase_contract::register_migration![
        mig_1762480990_create_permission_tables::Mig1762480990CreatePermissionTables,
        //
    ]
}
