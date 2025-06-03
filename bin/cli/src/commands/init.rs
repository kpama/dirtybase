use std::path::Path;

use crate::{
    content::{dump_stubs, make_directories},
    metadata::read_package_metadata,
};

pub(crate) fn init(package: Option<&String>) {
    let path_buf = if let Some(package) = package {
        read_package_metadata(package)
    } else {
        read_package_metadata("")
    };

    make_directories(&path_buf);
    dump_stubs(&path_buf);
    register_entry_file(&path_buf);
}

fn register_entry_file(path: &Path) {
    let files = ["lib.rs", "main.rs"];
    for a_file in files {
        let full_path = path.join(a_file);
        if full_path.exists() {
            if let Ok(mut content) = std::fs::read_to_string(&full_path) {
                if !content.contains("mod dirtybase_entry") {
                    content.insert_str(0, "pub mod dirtybase_entry;\n\r");
                    _ = std::fs::write(&full_path, content);
                }
            }
        }
    }
}
