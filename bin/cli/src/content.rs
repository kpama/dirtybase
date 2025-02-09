use std::{collections::HashMap, path::PathBuf};

pub(crate) fn directories() -> &'static [&'static str] {
    &[
        "dirtybase_entry/migration",
        "dirtybase_entry/event",
        "dirtybase_entry/event_handler",
        "dirtybase_entry/http",
        "dirtybase_entry/model",
    ]
}
pub(crate) fn files() -> &'static [&'static str] {
    &[
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
    ]
}

pub(crate) fn stubs<'a>() -> HashMap<&'a str, &'a str> {
    let mut file_content = HashMap::new();

    file_content.insert(
        "dirtybase_entry.rs",
        include_str!("./stubs/dirtybase_entry.stub.txt"),
    );
    file_content.insert(
        "dirtybase_entry/migration.rs",
        include_str!("./stubs/migration.stub.txt"),
    );

    file_content.insert(
        "dirtybase_entry/event_handler.rs",
        include_str!("./stubs/event_handler.stub.txt"),
    );

    file_content.insert(
        "new_migration",
        include_str!("./stubs/new_migration.stub.txt"),
    );

    file_content
}

pub(crate) fn make_directories(path_buf: &PathBuf) {
    for dir in directories() {
        let path = path_buf.join(dir);
        if !path.exists() {
            _ = std::fs::create_dir_all(path);
        }
    }
}

pub(crate) fn make_a_directory(path_buf: &PathBuf) {
    if !path_buf.exists() {
        _ = std::fs::create_dir_all(path_buf);
    }
}

pub(crate) fn dump_stubs(path_buf: &PathBuf) {
    let file_content = stubs();
    for filename in files() {
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

pub(crate) fn dump_a_stub(name: &str, path_buf: &PathBuf) {
    if let Some(stub) = stubs().get(name) {
        if !path_buf.exists() {
            _ = std::fs::write(&path_buf, stub);
        }
    }
}
