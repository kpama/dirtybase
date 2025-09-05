use std::process::Command;

pub(crate) fn read_package_metadata(package: &str) -> std::path::PathBuf {
    let mut cmd = Command::new("cargo");
    cmd.arg("metadata")
        .arg("--no-deps")
        .arg("--format-version=1");

    let output = cmd.output().unwrap();
    let out_string = std::str::from_utf8(&output.stdout).unwrap();
    let value: serde_json::Value = serde_json::from_str(out_string).unwrap();
    let packages = value.get("packages").unwrap().as_array().unwrap();
    let mut path = packages[0].get("targets").unwrap().as_array().unwrap()[0]
        .get("src_path")
        .unwrap();

    if !package.is_empty() {
        for a_package in packages {
            for a_target in a_package.get("targets").unwrap().as_array().unwrap() {
                if let Some(name) = a_target.get("name") {
                    if !name.is_string() {
                        continue;
                    }
                    // println!("checking: {} == {}", package, name.as_str().unwrap());
                    if package == name.as_str().unwrap() {
                        path = a_target.get("src_path").unwrap();
                    }
                }
            }
        }
    }

    if !package.is_empty() {
        let pass_name = package.to_lowercase();
        for pkg in packages {
            if let Some(value) = pkg.get("name")
                && pass_name == value.as_str().unwrap()
            {
                path = pkg.get("targets").unwrap().as_array().unwrap()[0]
                    .get("src_path")
                    .unwrap();
                break;
            }
        }
    }

    std::path::PathBuf::from(serde_json::from_value::<String>(path.clone()).unwrap())
        .parent()
        .unwrap()
        .to_path_buf()
}
