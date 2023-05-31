use std::env;

use dirtybase_db::base::{helper::generate_ulid, manager::Manager};

use crate::app::setup_database::COMPANY_TABLE;

pub struct CompanyEntity {
    pub internal_id: Option<u64>,
    pub id: String,
    pub name: String,
    pub description: String,
    pub core_user_id: String,
    pub creator: String,
    pub editor: Option<String>,
}

impl Default for CompanyEntity {
    fn default() -> Self {
        Self {
            internal_id: None,
            id: generate_ulid(),
            core_user_id: "".into(),
            name: "".into(),
            description: "".into(),
            creator: "".into(),
            editor: None,
        }
    }
}

impl CompanyEntity {
    pub fn from_env() -> Self {
        Self {
            name: if let Ok(name) = env::var("DTY_APP_NAME") {
                name
            } else {
                "Default Company".into()
            },
            ..Self::default()
        }
    }

    pub async fn exist(&self, manger: &mut Manager) -> bool {
        return !manger
            .select_from_table(COMPANY_TABLE, |q| {
                q.select("id");
                if !self.id.is_empty() {
                    q.eq("id", &self.id);
                } else {
                    q.eq("name", &self.name);
                }
            })
            .fetch_all_as_json()
            .await
            .is_empty();
    }
}
