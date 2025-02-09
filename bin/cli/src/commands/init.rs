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
}
