#[derive(Debug, serde::Deserialize)]
pub struct SwitchAppDto {
    pub app_id: String,
    pub role_id: String,
}
