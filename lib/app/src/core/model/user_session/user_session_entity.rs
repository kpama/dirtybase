use dirtybase_db::types::{DateTimeField, InternalIdField, NumberField, UlidField};

pub struct UserSessionEntity {
    pub internal_id: InternalIdField,
    pub id: UlidField,
    pub core_user_id: UlidField,
    pub ip: NumberField,
    pub user_agent: NumberField,
    pub impersonated_by: UlidField,
    pub logged_in_at: DateTimeField,
    pub expires_at: DateTimeField,
}
