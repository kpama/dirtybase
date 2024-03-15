#[derive(Debug, serde::Serialize)]
pub struct SwitchAppResultDto {
    token: String,
}

impl From<String> for SwitchAppResultDto {
    fn from(value: String) -> Self {
        Self { token: value }
    }
}

impl From<&str> for SwitchAppResultDto {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}
