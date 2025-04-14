use dirtybase_contract::{
    db_contract::{
        TableEntityTrait,
        base::manager::Manager,
        types::{
            JsonField, OptionalDateTimeField, OptionalStringField, OptionalTimestampField,
            TimestampField,
        },
    },
    session_contract::{SessionData, SessionId, SessionStorage},
};

use crate::SessionStorageResolver;

pub const NAME: &str = "database";

pub struct DatabaseStorage {
    manager: Manager,
}

impl DatabaseStorage {
    pub fn new(manager: Manager) -> Self {
        Self { manager }
    }

    pub async fn register() {
        SessionStorageResolver::register(NAME, resolver).await;
    }
}

#[async_trait::async_trait]
impl SessionStorage for DatabaseStorage {
    async fn store(&self, id: SessionId, value: SessionData) {
        let mut model = SessionTable::from(value);
        model.id = Some(id.to_string());

        let resullt = self
            .manager
            .upsert(
                SessionTable::table_name(),
                model,
                &[
                    SessionTable::col_name_for_data(),
                    SessionTable::col_name_for_expires(),
                ],
                &["id"],
            )
            .await;
        tracing::trace!("session store data: {:?}", resullt);
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
