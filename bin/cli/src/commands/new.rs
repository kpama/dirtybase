use std::process::Command;

use super::init;

pub(crate) fn create(path: &str) {
    _ = Command::new("cargo").arg("new").arg(path).output();
    let path_string = Some(path.to_string());
    // TODO: add dirtybase_contract and database_app crates
    init::init(path_string.as_ref());

    //FIXME: remove the message when we are able to auto add these crates
    println!("Please add these crates:");
    println!("\t dirtybase_app");
    println!("\t dirtybase_contract");
}
