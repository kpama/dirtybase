use std::process::Command;

use super::init;

pub(crate) fn create(path: &str) {
    _ = Command::new("cargo").arg("new").arg(path).output();
    let path_string = Some(path.to_string());
    init::init(path_string.as_ref());
}
