use super::entity::company::{setup_company_table, COMPANY_TABLE};
use dirtybase_db::{
    base::{helper::to_fk_column, manager::Manager},
    entity::user::{setup_users_table, USER_TABLE},
};

pub const APPLICATION_TABLE: &str = "core_app";
pub const APPLICATION_SCHEMA_TABLE: &str = "core_app_schema";
pub const ROLE_TABLE: &str = "core_app_role";
pub const ROLE_USER_TABLE: &str = "core_role_user";
pub const SYS_ADMIN_TABLE: &str = "core_sys_admin";
pub const MIGRATION_TABLE: &str = "core_migration";
pub const FILE_METADATA_TABLE: &str = "core_file_meta";

// The table that will hold company's tenets

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
    if !manager.has_table(USER_TABLE).await {
        log::error!("{} is require to create {} table", USER_TABLE, ROLE_TABLE);
    }

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

// System administrator table
async fn setup_sysadmins_table(manager: &Manager) {
    manager
        .create_table_schema(SYS_ADMIN_TABLE, |table| {
            table.ulid_fk(USER_TABLE, true);
            // status
            table.string("status");
            // blame
            table.blame();
            // timestamps
            table.timestamps();
            // soft delete
            table.soft_deletable();
        })
        .await;
}

// The table that will hold migration information
async fn setup_migration_table(manager: &Manager) {
    manager
        .create_table_schema(MIGRATION_TABLE, |table| {
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
    manager
        .create_table_schema(FILE_METADATA_TABLE, |table| {
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

async fn create_default_records(mut manager: &mut Manager) {
    // default user
    // let mut user = UserEntity::from_env();
    // if !user.exist(&mut manager).await {
    //     user.id = generate_ulid();
    //     let mut default_user = HashMap::<String, String>::new();
    //     default_user.insert("id".into(), generate_ulid());
    //     default_user.insert("username".into(), user.username.clone());
    //     default_user.insert("email".into(), user.email.clone());

    //     if !user.hashed_password().is_empty() {
    //         default_user.insert("password".into(), user.hashed_password());
    //     }

    //     manager.insert_record(USER_TABLE, default_user).await;
    // }

    // // default company
    // let company = CompanyEntity::from_env();
    // if !company.exist(manager).await {
    //     let default_company = InsertValueBuilder::new()
    //         .add("id", generate_ulid())
    //         .add("name", &company.name)
    //         .add("creator", generate_ulid())
    //         .build();

    //     manager.insert_record(COMPANY_TABLE, default_company).await;
    //     println!("{:?}", &company.name)
    // }

    // default app
    // default roles
}

pub(crate) async fn create_data_tables(mut manager: Manager) {
    setup_users_table(&manager).await;
    setup_migration_table(&manager).await;
    setup_file_metadata_table(&manager).await;
    setup_company_table(&manager).await;
    setup_applications_table(&manager).await;
    setup_schema_table(&manager).await;
    setup_roles_table(&manager).await;
    setup_role_users_table(&manager).await;
    setup_sysadmins_table(&manager).await;
    create_default_records(&mut manager).await;
}
