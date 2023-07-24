use busybody::ServiceContainer;
use dirtybase_db::{
    base::helper::generate_ulid,
    dirtybase_db_types::types::{DateTimeField, InternalIdField, StringField, UlidField},
    entity::user::UserEntity,
    macros::DirtyTable,
};

#[derive(Debug, Clone, Default, DirtyTable, serde::Serialize, serde::Deserialize)]
#[dirty(table = "core_company")]
pub struct CompanyEntity {
    pub internal_id: InternalIdField,
    pub id: UlidField,
    pub name: StringField,
    pub description: StringField,
    pub core_user_id: UlidField,
    #[dirty(col = "creator_id", skip_select)]
    pub creator: Option<UserEntity>,
    #[dirty(skip_select)]
    pub creator_id: UlidField,
    pub editor_id: UlidField,
    pub created_at: DateTimeField,
    pub updated_at: DateTimeField,
    pub deleted_at: DateTimeField,
}

#[busybody::async_trait]
impl busybody::Injectable for CompanyEntity {
    async fn inject(_: &ServiceContainer) -> Self {
        Self::default()
    }
}

impl CompanyEntity {
    pub fn new() -> Self {
        Self {
            id: Some(generate_ulid()),
            ..Self::default()
        }
    }
}
