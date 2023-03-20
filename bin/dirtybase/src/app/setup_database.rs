use dirtybase_db::base::{
    manager::Manager,
    to_fk_column,
    user_table::{setup_users_table, user_table_name, USER_TABLE},
};

pub const COMPANY_TABLE: &str = "_core_company";
pub const APPLICATION_TABLE: &str = "_core_app";
pub const APPLICATION_SCHEMA_TABLE: &str = "_core_app_schema";
pub const ROLE_TABLE: &str = "_core_app_role";
pub const ROLE_USER_TABLE: &str = "_core_role_user";

// The table that will hold company's tenets
async fn setup_company_table(manager: &Manager) {
    manager
        .create_table_schema(COMPANY_TABLE, |table| {
            let user_table_name = user_table_name();
            // internal_id
            // id
            table.id_set();
            // name
            table.string("name");
            // owner
            table.ulid_fk(&user_table_name, false).set_is_nullable(true);
            // description
            table.sized_string("description", 512).set_is_nullable(true);
            // blame
            table.blame();
            // timestamp
            table.timestamps();
            // soft delete
            table.soft_deletable();
        })
        .await;
}

async fn setup_applications_table(manager: &Manager) {
    manager
        .create_table_schema(APPLICATION_TABLE, |table| {
            // internal_id
            // id
            table.id_set();
            // company_id
            table.ulid_fk(COMPANY_TABLE, true);
            // name
            table.string("name");
            //description
            table.sized_string("description", 512).set_is_nullable(true);
            // blame
            table.blame();
            // timestamp
            table.timestamps();
            // soft delete
            table.soft_deletable();
        })
        .await
}

// The table that will contain the "collections" definitions
async fn setup_schema_table(manager: &Manager) {
    manager
        .create_table_schema(APPLICATION_SCHEMA_TABLE, |table| {
            // internal_id
            // id
            table.id_set();
            // application ID
            table.ulid_fk(APPLICATION_TABLE, true);
            // table/collection name
            table.string("table_name");
            // table/collection definition
            table.json("table_definition");
            // blame
            table.blame();
            // timestamp
            table.timestamps();
        })
        .await
}

// The global roles table
async fn setup_roles_table(manager: &Manager) {
    manager
        .create_table_schema(ROLE_TABLE, |table| {
            // internal_id
            // id
            table.id_set();
            // application
            table.ulid_fk(APPLICATION_TABLE, true);
            // name
            table.string("name");
            // blame
            table.blame();
            // timestamps
            table.timestamps();
            // soft delete
            table.soft_deletable();
        })
        .await;
}

// A user role
async fn setup_role_users_table(manager: &Manager) {
    manager
        .create_table_schema(ROLE_USER_TABLE, |table| {
            // role id
            table.ulid_fk(ROLE_TABLE, true);
            // user id
            table.ulid_fk(USER_TABLE, true);
            // blame
            table.blame();
            // timestamps
            table.timestamps();

            // primary key
            let keys = vec![to_fk_column(ROLE_TABLE), to_fk_column(USER_TABLE)];
            table.primary_index(
                keys.iter()
                    .map(AsRef::as_ref)
                    .collect::<Vec<&str>>()
                    .as_slice(),
            );
        })
        .await;
}

// The table that will hold migration information
async fn setup_migration_table(manager: &Manager) {
    let name = "_core_migration";
    manager
        .create_table_schema(name, |table| {
            // id
            table.id(None);
            // migration name
            table.string("name");
            // created at
            table.created_at();
            // deleted at
            table.updated_at();
        })
        .await;
}

// The table that will hold file metadata
async fn setup_file_metadata_table(manager: &Manager) {
    let name = "_core_file_meta";
    manager
        .create_table_schema(name, |table| {
            // internal_id
            // id
            table.id_set();
            // external_id
            table.ulid("external_id").set_is_nullable(false);
            // meta
            table.json("meta");
            // timestamp
            table.timestamps();
        })
        .await;
}

pub(crate) async fn create_data_tables(manager: Manager) {
    setup_users_table(&manager).await;
    setup_migration_table(&manager).await;
    setup_file_metadata_table(&manager).await;
    setup_company_table(&manager).await;
    setup_roles_table(&manager).await;
    setup_role_users_table(&manager).await;
    setup_applications_table(&manager).await;
    setup_schema_table(&manager).await;
}
