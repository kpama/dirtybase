use busybody::Injectable;

use crate::core::{
    model::dirtybase_user::dirtybase_user_helpers::authentication_error_status::AuthenticationErrorStatus,
    pipeline::user_login_pipeline::register_pipes,
};

// TODO: remove serde::Serialization. This data should never leave the backend
/// Incoming DTO when the user is attempting to login
#[derive(Default, Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct UserLoginPayload {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: String,
}

impl UserLoginPayload {
    pub(crate) fn validate(&self) -> Result<bool, AuthenticationErrorStatus> {
        if (self.username.is_none() || self.username.as_ref().unwrap().is_empty())
            && (self.email.is_none() || self.email.as_ref().unwrap().is_empty())
        {
            return Err(AuthenticationErrorStatus::CredentialIncorrect);
        }

        if self.password.is_empty() {
            return Err(AuthenticationErrorStatus::PasswordIncorrect);
        }

        Ok(true)
    }
}

#[busybody::async_trait]
impl Injectable for UserLoginPayload {
    async fn inject(container: &busybody::ServiceContainer) -> Self {
        container.proxy_value().unwrap_or_default()
    }
}

#[busybody::async_trait]
impl fama::PipelineBuilderTrait for UserLoginPayload {
    async fn setup_pipeline_builder(
        builder: fama::PipelineBuilder<Self>,
    ) -> fama::PipelineBuilder<Self> {
        register_pipes(builder).await
    }
}
