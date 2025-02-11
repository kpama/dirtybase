use crate::db::types::ArcUuid7;

pub const GLOBAL_APP_CONTEXT_ID: &str = "0194d479-fee1-7a81-8c20-5b8726efddf0";

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AppContext {
    id: ArcUuid7,
}

impl AppContext {
    pub fn make_global() -> Self {
        Self {
            id: ArcUuid7::try_from(GLOBAL_APP_CONTEXT_ID).unwrap(),
        }
    }

    pub fn id(&self) -> ArcUuid7 {
        self.id.clone()
    }
}
