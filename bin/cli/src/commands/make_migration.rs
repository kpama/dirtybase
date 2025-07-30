use std::process::Command;

use crate::{
    content::{dump_a_stub, make_a_directory, read_entry_file, stubs, update_entry_file},
    metadata::read_package_metadata,
};

pub fn make(package: Option<&String>, name: &str) {
    let path_buf = if let Some(package) = package {
        read_package_metadata(package)
    } else {
        read_package_metadata("")
    };

    let ts = chrono::Utc::now().timestamp();

    let filename = name.split_whitespace().collect::<String>().to_lowercase();

    let module_name = format!("mig_{}_{}", ts, &filename);
    let struct_name = format!(
        "Mig{}{}",
        ts,
        dirtybase_helper::cruet::case::to_pascal_case(name)
    );

    let built = stubs()
        .get("new_migration")
        .unwrap()
        .replace("struct_name", &struct_name);

    let migration_dir = path_buf.join("dirtybase_entry").join("migration");
    make_a_directory(&migration_dir);

    let mod_path = path_buf.join("dirtybase_entry").join("migration.rs");
    dump_a_stub("dirtybase_entry/migration.rs", &mod_path);
    dump_a_stub("dirtybase_entry.rs", &path_buf);

    let path = path_buf
        .join("dirtybase_entry")
        .join("migration")
        .join(format!("{module_name}.rs"));

    let mut module = std::fs::read_to_string(&mod_path).unwrap();

    module = module.replace(
        "register_migration![",
        format!("register_migration![\n{module_name}::{struct_name},").as_str(),
    );

    _ = std::fs::write(&mod_path, format!("mod {}; \n{}", &module_name, module));

    _ = std::fs::write(&path, built);

    if let Ok(mut entry_content) = read_entry_file(&path_buf) {
        if !entry_content.contains("mod migration;") {
            entry_content.insert_str(0, "mod migration;\r\n");
            _ = update_entry_file(&path_buf, entry_content);
        }
    }

    if package.is_some() {
        _ = Command::new(format!("cargo -p {}", package.as_ref().unwrap()))
            .arg("fmt")
            .arg("--all")
            .output();
    } else {
        _ = Command::new("cargo").arg("fmt").arg("--all").output();
    }
}
