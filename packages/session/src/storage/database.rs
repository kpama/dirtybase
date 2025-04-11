use std::collections::HashMap;

use dirtybase_contract::{
    db_contract::{
        base::manager::Manager,
        field_values::FieldValue,
        types::{
            JsonField, OptionalDateTimeField, OptionalStringField, OptionalTimestampField,
            TimestampField,
        },
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
        // 1. Try to update existing data
        if let Ok(Some(_)) = self
            .manager
            .select_from::<SessionTable>(|q| {
                q.eq("id", id.clone().to_string());
            })
            .fetch_one_to::<SessionTable>()
            .await
        {
            // let result = self
            //     .manager
            //     .delete_from_table::<SessionTable>(|q| {
            //         q.eq(SessionTable::col_name_for_id(), id.clone().to_string());
            //     })
            //     .await;
            let mut data = HashMap::new();
            let model: SessionTable = value.into();
            data.insert(
                SessionTable::col_name_for_expires().to_string(),
                FieldValue::from(model.expires),
            );
            data.insert(
                SessionTable::col_name_for_data().to_string(),
                FieldValue::from(model.data),
            );

            let result = self
                .manager
                .update_table::<SessionTable>(data, |q| {
                    //
                    q.eq("id", id.to_string());
                })
                .await;
        } else {
            let mut model: SessionTable = value.into();
            model.id = Some(id.to_string());
            let result = self.manager.insert_into::<SessionTable>(model).await;
        }
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
            Ok(Some(data)) => SessionData::from(data),
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
    #[dirty(skip_insert)]
    created_at: OptionalDateTimeField,
    expires: OptionalTimestampField,
}

impl From<SessionTable> for SessionData {
    fn from(value: SessionTable) -> Self {
        SessionData::new_from(value.data, value.expires.unwrap_or_default().timestamp())
    }
}

impl From<SessionData> for SessionTable {
    fn from(value: SessionData) -> Self {
        Self {
            id: None,
            data: value.all(),
            created_at: None,
            expires: TimestampField::from_timestamp(value.expires(), 0),
        }
    }
}

pub async fn resolver(mut resolver: SessionStorageResolver) -> SessionStorageResolver {
    if let Ok(manager) = resolver.context_ref().get::<Manager>().await {
        resolver.set_storage(DatabaseStorage::new(manager));
    }
    resolver
}
