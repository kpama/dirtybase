use std::process::Command;

pub(crate) fn read_package_metadata(package: &str) -> std::path::PathBuf {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--no-deps")
        .arg("--format-version=1")
        .output();

    let out = output.unwrap().stdout;
    let o = std::str::from_utf8(&out).unwrap();
    let value: serde_json::Value = serde_json::from_str(o).unwrap();

    let packages = value.get("packages").unwrap().as_array().unwrap();

    let mut path = packages[0].get("targets").unwrap().as_array().unwrap()[0]
        .get("src_path")
        .unwrap();

    if !package.is_empty() {
        let pass_name = package.to_lowercase();
        for pkg in packages {
            if let Some(value) = pkg.get("name") {
                if pass_name == value.as_str().unwrap() {
                    path = pkg.get("targets").unwrap().as_array().unwrap()[0]
                        .get("src_path")
                        .unwrap();
                    break;
                }
            }
        }
    }

    std::path::PathBuf::from(serde_json::from_value::<String>(path.clone()).unwrap())
        .parent()
        .unwrap()
        .to_path_buf()
}
