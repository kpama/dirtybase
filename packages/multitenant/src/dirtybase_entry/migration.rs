mod mig_1767333281_create_tenant_table;
/**
 * The following function is automatically modified
 * do not manually edit it
 */
pub(crate) fn setup() -> Option<dirtybase_contract::ExtensionMigrations> {
    dirtybase_contract::register_migration![
        mig_1767333281_create_tenant_table::Mig1767333281CreateTenantTable,
        //
    ]
}
