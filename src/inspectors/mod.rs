mod angular;
mod e2e_test;
mod file_type;
mod inspector;
mod package_json;
mod unit_test;
mod utils;

pub use angular::AngularInspector;
pub use e2e_test::EndToEndTestInspector;
pub use file_type::FileTypeInspector;
pub use inspector::{FileInspector, FileInspectorOptions};
pub use package_json::PackageJsonInspector;
pub use unit_test::UnitTestInspector;
