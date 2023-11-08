use std::process::Command;

const MIGRATION_STUB: &str = "
use dirtybase_contract::db::migration::Migration;
use dirtybase_contract::db::base::manager::Manager;

pub struct struct_name;

#[dirtybase_contract::async_trait]
impl Migration for struct_name {
  async fn up(&self, manager: &Manager) {
     println!(\"This is a test going up\");
  }

  async fn down(&self, manager: &Manager) {
     println!(\"This is a test going down\");
  }
}
";

pub fn make(name: &str, path_buf: &std::path::PathBuf) {
    let ts = chrono::Utc::now().timestamp();

    let filename = name.split_whitespace().collect::<String>().to_lowercase();

    let module_name = format!("mig_{}_{}", ts, filename);
    let struct_name = format!("Mig{}{}", ts, filename.split('_').collect::<String>());

    let built = MIGRATION_STUB.replace("struct_name", &struct_name);

    // TODO: Check if migration folder exist
    let migration_dir = path_buf.join("dirtybase_entry").join("migration");
    if !migration_dir.exists() {
        _ = std::fs::create_dir_all(migration_dir);
    }
    // TODO: Check if migration module exist
    let mod_path = path_buf.join("dirtybase_entry").join("migration.rs");
    if !mod_path.exists() {
        _ = std::fs::write(&mod_path, "");
    }

    let path = path_buf
        .join("dirtybase_entry")
        .join("migration")
        .join(format!("{}.rs", module_name));

    println!("new file path: {:?}", path.to_str());

    let mut module = std::fs::read_to_string(&mod_path).unwrap();

    // FIXME: Use something more robust to generate the rust code
    module = module.replace(
        "// dty_inject",
        format!(
            "Box::new({}::{}),\n // dty_inject",
            module_name, struct_name
        )
        .as_str(),
    );

    _ = std::fs::write(&mod_path, format!("mod {}; \n{}", &module_name, module));

    _ = std::fs::write(&path, built);

    _ = Command::new("cargo").arg("fmt").arg("--all").output();
}
