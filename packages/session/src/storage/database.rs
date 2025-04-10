use std::collections::HashMap;

use dirtybase_contract::{
    db_contract::{
        TableEntityTrait,
        base::manager::Manager,
        types::{DateTimeField, JsonField, OptionalDateTimeField, OptionalStringField},
    },
    session_contract::{SessionData, SessionId, SessionStorage},
};

use crate::SessionStorageResolver;

pub const NAME: &'static str = "database";

pub struct DatabaseStorage {
    manager: Manager,
}

impl DatabaseStorage {
    pub fn new(manager: Manager) -> Self {
        Self { manager }
    }
}

#[async_trait::async_trait]
impl SessionStorage for DatabaseStorage {
    async fn store(&self, id: SessionId, value: SessionData) {
        log::debug!("db session storage store");
        // 1. Try to update existing data
        let result = self
            .manager
            .delete_from_table::<SessionTable>(|q| {
                q.eq(SessionTable::col_name_for_id(), id.clone().to_string());
            })
            .await;
        tracing::warn!("deleted session {}, result: {:?}", &id, result);
        let mut model: SessionTable = value.into();
        model.id = Some(id.to_string());
        let result = self.manager.insert_into::<SessionTable>(model).await;
        tracing::warn!("inserted session {}, result: {:?}", &id, result);
    }

    async fn get(&self, id: &SessionId) -> SessionData {
        println!("select * from sessions where id = '{}'", id);

        match self
            .manager
            .select_from::<SessionTable>(|query| {
                query.eq(SessionTable::col_name_for_id(), id.to_string());
            })
            .fetch_one_to::<SessionTable>()
            .await
        {
            Ok(Some(data)) => {
                tracing::warn!("got the existing session data: {:#?}", &data);
                let x = SessionData::from(data);
                tracing::warn!("got the existing session data: {:#?}", &x);
                x
            }
            _ => {
                tracing::error!("we should have the session in storage");
                SessionData::new()
            }
        }
    }

    async fn remove(&self, _id: &SessionId) -> Option<SessionData> {
        log::debug!("db session storage remove");

        None
    }
    async fn gc(&self, _lifetime: i64) {
        log::debug!("db session storage clean expired");
    }
}

#[derive(Debug, dirtybase_db_macro::DirtyTable, Default, Clone)]
#[dirty(table = "sessions")]
pub struct SessionTable {
    id: OptionalStringField,
    data: JsonField,
    // data: HashMap<String, String>,
    created_at: OptionalDateTimeField,
    updated_at: OptionalDateTimeField,
}

impl From<SessionTable> for SessionData {
    fn from(value: SessionTable) -> Self {
        let mut data = HashMap::<String, String>::new();
        for (key, value) in value.data {
            data.insert(key, value.to_string());
        }
        SessionData::new_from(
            data,
            value.created_at.unwrap_or_default().timestamp(),
            value.updated_at.unwrap_or_default().timestamp(),
        )
    }
}

impl From<SessionData> for SessionTable {
    fn from(value: SessionData) -> Self {
        let mut data = serde_json::Map::new();
        for (key, value) in value.all() {
            data.insert(key, value.to_string().into());
        }

        Self {
            id: None,
            data,
            created_at: None,
            updated_at: None,
        }
    }
}

pub async fn resolver(mut resolver: SessionStorageResolver) -> SessionStorageResolver {
    if let Ok(manager) = resolver.context_ref().get::<Manager>().await {
        resolver.set_storage(DatabaseStorage::new(manager));
    }
    resolver
}
