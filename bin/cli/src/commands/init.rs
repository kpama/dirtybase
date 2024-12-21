use std::collections::HashMap;

use crate::metadata::read_package_metadata;

pub(crate) fn init(package: Option<&String>) {
    let path_buf = if let Some(package) = package {
        read_package_metadata(package)
    } else {
        read_package_metadata("")
    };

    let directories = [
        "dirtybase_entry/migration",
        "dirtybase_entry/event",
        "dirtybase_entry/event_handler",
        "dirtybase_entry/http",
        "dirtybase_entry/model",
    ];

    let files = [
        // migration
        "dirtybase_entry/migration/.gitkeep",
        "dirtybase_entry/migration.rs",
        // event
        "dirtybase_entry/event/.gitkeep",
        "dirtybase_entry/event.rs",
        // event handler
        "dirtybase_entry/event_handler/.gitkeep",
        "dirtybase_entry/event_handler.rs",
        // http
        "dirtybase_entry/http/.gitkeep",
        "dirtybase_entry/http.rs",
        // model
        "dirtybase_entry/model/.gitkeep",
        "dirtybase_entry/model.rs",
        // setup,
        "dirtybase_entry.rs",
    ];

    let mut file_content = HashMap::new();

    file_content.insert(
        "dirtybase_entry.rs",
        include_str!("../stubs/dirtybase_entry.stub.rs"),
    );
    file_content.insert(
        "dirtybase_entry/migration.rs",
        include_str!("../stubs/migration.stub.rs"),
    );

    file_content.insert(
        "dirtybase_entry/event_handler.rs",
        include_str!("../stubs/event_handler.stub.rs"),
    );

    for dir in directories {
        let path = path_buf.join(dir);
        if !path.exists() {
            _ = std::fs::create_dir_all(path);
        }
    }

    for filename in files {
        let path = path_buf.join(filename);

        if !path.exists() && !path.is_dir() {
            let content = if let Some(content) = file_content.get(filename) {
                content
            } else {
                ""
            };
            _ = std::fs::write(path, content);
        }
    }
}
