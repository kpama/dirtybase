mod adapter_manager;
mod email;
mod mailman;

use adapters::dummy_adapter::DummyAdapter;
use adapters::failover_adapter::FailoverAdapter;
use adapters::smtp_adapter::SmtpAdapter;
use adapters::test_adapter::TestAdapter;

pub mod adapters;

pub use adapter_manager::AdapterTrait;
pub use email::Email;
pub use email::EmailBuilder;
pub use mailman::Mailman;

pub async fn register_mail_adapters() {
    // TODO: Use configuration here....
    TestAdapter.register().await;
    DummyAdapter.register().await;
    SmtpAdapter.register().await;
    FailoverAdapter.register().await;
}
