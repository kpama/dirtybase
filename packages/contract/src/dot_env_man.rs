pub struct DotEnvManipulator {
    path: String,
    lines: Vec<String>,
}

impl DotEnvManipulator {
    pub fn new(path: Option<&str>) -> Result<Self, std::io::Error> {
        let path = path.unwrap_or(".env").to_string();
        let content = std::fs::read_to_string(&path)?;
        Ok(Self {
            lines: content.lines().map(String::from).collect::<Vec<String>>(),
            path,
        })
    }

    pub fn value(&self, key: &str) -> Option<String> {
        for entry in &self.lines {
            if entry.contains(key) {
                if let Some((_, v)) = entry.split_once("=") {
                    if !v.is_empty() {
                        return Some(
                            v.split_once("#")
                                .unwrap()
                                .0
                                .replace("\"", "")
                                .trim()
                                .to_string(),
                        );
                    }
                }
            }
        }

        None
    }

    pub fn set_quote_value(&mut self, key: &str, value: &str) {
        self.set_value(key, &format!("\"{value}\""));
    }

    pub fn set_value(&mut self, key: &str, value: &str) {
        for entry in &mut self.lines {
            if entry.contains(key) {
                if let Some((_, v)) = entry.split_once("=") {
                    if !v.trim().is_empty() {
                        let old = v.split_once("#").unwrap().0.trim().to_string();
                        if old == "\"\"" {
                            *entry = entry.replace("\"\"", value);
                        } else if !old.is_empty() {
                            *entry = entry.replace(&old, value);
                        } else {
                            *entry = entry.replace("=", &format!("={value}"));
                        }
                    } else {
                        *entry = entry.replace("=", &format!("={value}"));
                    }
                }
                break;
            }
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        std::fs::write(&self.path, self.lines.join("\n"))
    }
}
