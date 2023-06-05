use super::entity::app::{AppRepository, AppService};
use super::entity::company::{CompanyRepository, CompanyService};
use super::entity::role::{RoleRepository, RoleService};
use super::entity::role_user::{RoleUserRepository, RoleUserService};
use super::entity::sys_admin::{SysAdminRepository, SysAdminService};
use super::setup_database::create_data_tables;
use super::setup_defaults::setup_default_entities;
use super::Config;
use actix_web::web;
use dirtybase_db::base::connection::ConnectionPoolRegisterTrait;
use dirtybase_db::base::manager::Manager;
use dirtybase_db::driver::mysql::mysql_pool_manager::MySqlPoolManagerRegisterer;
use dirtybase_db::driver::sqlite::sqlite_pool_manager::SqlitePoolManagerRegisterer;
use dirtybase_db::entity::user::{UserRepository, UserService};
use dirtybase_db::ConnectionPoolManager;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use jwt::VerifyWithKey;
use sha2::Sha256;
use sqlx::{any::AnyKind, mysql::MySqlPoolOptions, MySql, Pool};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug)]
pub struct DirtyBase {
    default_db: String,
    hmac_key: Option<Hmac<Sha256>>,
    config: Config,
    pool_manager: ConnectionPoolManager,
}

pub type DirtyBaseWeb = web::Data<DirtyBase>;

impl DirtyBase {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let mut connection_pools: Vec<Box<dyn ConnectionPoolRegisterTrait>> = Vec::new();
        let default;

        match AnyKind::from_str(config.db_connection()) {
            Ok(kind) => match kind {
                AnyKind::MySql => {
                    connection_pools.push(Box::new(MySqlPoolManagerRegisterer));
                    default = "mysql"
                }
                AnyKind::Sqlite => {
                    connection_pools.push(Box::new(SqlitePoolManagerRegisterer));
                    default = "sqlite"
                }
            },
            Err(_) => panic!("Could not determine database kind"),
        }

        let pool_manager = ConnectionPoolManager::new(
            connection_pools,
            default,
            &config.db_connection(),
            config.max_db_pool(),
        )
        .await;

        let instance = Self {
            default_db: default.into(),
            hmac_key: match Hmac::new_from_slice(config.secret().as_bytes()) {
                Ok(key) => Some(key),
                Err(_) => None,
            },
            pool_manager,
            config,
        };

        Ok(instance)
    }

    pub fn default_db(&self) -> &String {
        &self.default_db
    }

    pub fn schema_manger(&self) -> Manager {
        self.pool_manager.default_schema_manager().unwrap()
    }

    pub fn user_service(&self) -> UserService {
        UserService::new(UserRepository::new(self.schema_manger()))
    }

    pub fn company_service(&self) -> CompanyService {
        CompanyService::new(CompanyRepository::new(self.schema_manger()))
    }

    pub fn app_service(&self) -> AppService {
        AppService::new(AppRepository::new(self.schema_manger()))
    }

    pub fn role_service(&self) -> RoleService {
        RoleService::new(RoleRepository::new(self.schema_manger()))
    }

    pub fn sys_admin_service(&self) -> SysAdminService {
        SysAdminService::new(SysAdminRepository::new(self.schema_manger()))
    }

    pub fn role_user_service(&self) -> RoleUserService {
        RoleUserService::new(RoleUserRepository::new(self.schema_manger()))
    }

    pub async fn db_setup(&self) {
        create_data_tables(self.schema_manger()).await;
        setup_default_entities(self).await;
    }

    pub fn hmac_key(&self) -> &Option<Hmac<Sha256>> {
        &self.hmac_key
    }

    pub fn config(&self) -> Config {
        self.config.clone()
    }

    pub fn sign_to_jwt(&self, claims: HashMap<String, String>) -> Option<String> {
        if let Some(key) = self.hmac_key() {
            return match claims.sign_with_key(key) {
                Ok(s) => Some(s),
                Err(e) => {
                    log::error!("could not generate jwt: {}", e.to_string());
                    None
                }
            };
        }

        None
    }

    pub fn verify_jwt(&self, jwt: &str) -> Option<HashMap<String, String>> {
        if let Some(key) = self.hmac_key() {
            let result: Result<HashMap<String, String>, _> = jwt.verify_with_key(key);
            return match result {
                Ok(claim) => Some(claim),
                Err(e) => {
                    log::info!("Could not verify JWT: {}", e.to_string());
                    None
                }
            };
        }
        None
    }
}

pub async fn db_connect(conn: &str, max_connection: u32) -> anyhow::Result<Pool<MySql>> {
    match MySqlPoolOptions::new()
        .max_connections(max_connection)
        .connect(conn)
        .await
    {
        Ok(conn) => {
            log::info!("Maximum DB pool connection: {}", max_connection);
            Ok(conn)
        }
        Err(e) => {
            log::error!("could not connect to the database: {:#?}", &e);
            Err(anyhow::anyhow!(
                "could not connect to the database: {:#?}",
                e
            ))
        }
    }
}
