#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct UserConfig {
    #[serde(default)]
    enable: bool,
}
