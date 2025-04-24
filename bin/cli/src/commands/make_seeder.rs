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

    let filename = name.split_whitespace().collect::<String>().to_lowercase();

    let module_name = format!("{}_seeder", &filename);

    let seeder_dir = path_buf.join("dirtybase_entry").join("seeder");
    make_a_directory(&seeder_dir);

    let mod_path = path_buf.join("dirtybase_entry").join("seeder.rs");
    dump_a_stub("dirtybase_entry/seeder.rs", &mod_path);
    dump_a_stub("dirtybase_entry.rs", &path_buf);

    let path = path_buf
        .join("dirtybase_entry")
        .join("seeder")
        .join(format!("{}.rs", module_name));
    let built = stubs()
        .get("new_seeder")
        .unwrap()
        .replace("seeder_name", &name);

    let mut module = std::fs::read_to_string(&mod_path).unwrap();
    let seed_function = format!(
        r#"register_seeders() {{
           SeederRegisterer::register("{}", |manager| {{
                Box::pin(async move {{ {}::seed(manager).await }})
    }}).await;
        "#,
        name, module_name
    );

    module = module.replace("register_seeders() {", &seed_function);
    module = module.replace("register_seeders(){", &seed_function);
    module = module.replace("register_seeders()\n{", &seed_function);

    _ = std::fs::write(&mod_path, format!("mod {}; \n{}", &module_name, module));
    _ = std::fs::write(&path, built);

    if let Ok(mut entry_content) = read_entry_file(&path_buf) {
        if !entry_content.contains("mod seeder;") {
            entry_content.insert_str(0, "mod seeder;\r\n");
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
