mod angular;
mod file_type;
mod inspector;
mod package_json;
mod test;
mod utils;

pub use angular::AngularInspector;
pub use file_type::FileTypeInspector;
pub use inspector::{FileInspector, FileInspectorOptions};
pub use package_json::PackageJsonInspector;
pub use test::TestInspector;
