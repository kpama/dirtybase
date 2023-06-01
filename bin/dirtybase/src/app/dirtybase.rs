use super::setup_database::create_data_tables;
use super::Config;
use actix_web::web;
use dirtybase_db::base::schema::RelationalDbTrait;
use dirtybase_db::{base, driver::mysql::mysql_schema_manager::MySqlSchemaManager};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use jwt::VerifyWithKey;
use sha2::Sha256;
use sqlx::{any::AnyKind, mysql::MySqlPoolOptions, MySql, Pool};
use std::collections::HashMap;
use std::{str::FromStr, sync::Arc};

#[derive(Debug)]
pub struct DirtyBase {
    db_pool: Arc<Pool<MySql>>,
    kind: AnyKind,
    hmac_key: Option<Hmac<Sha256>>,
    config: Config,
}

pub type DirtyBaseWeb = web::Data<DirtyBase>;

impl DirtyBase {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let kind = AnyKind::from_str(config.db_connection()).unwrap_or(AnyKind::MySql);

        let instance = Self {
            kind,
            db_pool: Arc::new(db_connect(config.db_connection(), config.max_db_pool()).await?),
            hmac_key: match Hmac::new_from_slice(config.secret().as_bytes()) {
                Ok(key) => Some(key),
                Err(_) => None,
            },
            config,
        };

        Ok(instance)
    }

    pub fn kind(&self) -> &AnyKind {
        &self.kind
    }

    pub fn schema_manger(&self) -> base::manager::Manager {
        let db = MySqlSchemaManager::instance(self.db_pool.clone());
        base::manager::Manager::new(Box::new(db))
    }

    pub async fn db_setup(&self) {
        create_data_tables(self.schema_manger()).await;
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
