use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct AppCancellationToken(CancellationToken);

impl AppCancellationToken {
    pub fn new(token: CancellationToken) -> Self {
        Self(token)
    }

    pub fn into_inner(self) -> CancellationToken {
        self.0
    }
}

impl From<AppCancellationToken> for CancellationToken {
    fn from(value: AppCancellationToken) -> Self {
        value.0
    }
}
