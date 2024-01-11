mod audit_log_entity;
mod audit_log_repository;
mod audit_log_service;

pub use audit_log_entity::AuditLogEntity;
pub use audit_log_repository::AuditLogRepository;
pub use audit_log_service::AuditLogService;
use dirtybase_db::base::{manager::Manager, table::CREATED_AT_FIELD};
use dirtybase_db::TableEntityTrait;

pub const AUDIT_LOG_TABLE: &str = "core_audit_log";

pub const AUDIT_LOG_TABLE_SUBJECT: &str = "subject_id";
pub const AUDIT_LOG_TABLE_EVENT: &str = "event";
pub const AUDIT_LOG_TABLE_META: &str = "meta";
pub const AUDIT_LOG_TABLE_RECORD: &str = "record";
pub const AUDIT_LOG_TABLE_CREATED_AT_FIELD: &str = CREATED_AT_FIELD;

pub async fn setup_audit_log_table(manager: &Manager) {
    manager
        .create_table_schema(AuditLogEntity::table_name(), |table| {
            table.ulid(AUDIT_LOG_TABLE_SUBJECT).set_is_nullable(false);

            // Event or log title
            table.string(AUDIT_LOG_TABLE_EVENT).set_is_nullable(false);

            // Additional metadata
            table.json(AUDIT_LOG_TABLE_META).set_is_nullable(true);

            // The record as JSON
            table.json(AUDIT_LOG_TABLE_RECORD).set_is_nullable(true);

            // logged datetime
            table.created_at();

            // indexes
            table.index(&[AUDIT_LOG_TABLE_SUBJECT]);
        })
        .await;
}
