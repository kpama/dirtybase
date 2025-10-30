mod mig_1758251736_createlocktable;

/**
 * The following function is automatically modified
 * do not manually edit it
 */
pub(crate) fn setup() -> Option<dirtybase_contract::ExtensionMigrations> {
    dirtybase_contract::register_migration![
        mig_1758251736_createlocktable::Mig1758251736CreateLockTable,
        //
    ]
}
