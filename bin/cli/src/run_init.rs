pub fn init(path_buf: &std::path::PathBuf) {
    let directories = ["migration", "event", "event_handler", "http", "model"];

    let files = [
        // migration
        "migration/.gitkeep",
        "migration.rs",
        // event
        "event/.gitkeep",
        "event.rs",
        // event handler
        "event_handler/.gitkeep",
        "event_handler.rs",
        // http
        "http/.gitkeep",
        "http.rs",
        // model
        "model/.gitkeep",
        "model.rs",
        // setup,
        "dirtybase_setup.rs",
    ];

    for dir in directories {
        let path = path_buf.join(dir);
        if !path.exists() {
            _ = std::fs::create_dir_all(path);
        }
    }

    for filename in files {
        let path = path_buf.join(filename);

        if !path.exists() && !path.is_dir() {
            _ = std::fs::write(path, "");
        }
    }
}
