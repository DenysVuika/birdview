mod angular;
mod inspector;
mod package;
mod test;
mod utils;

pub use angular::AngularInspector;
pub use inspector::{FileInspector, FileInspectorOptions};
pub use package::PackageJsonInspector;
pub use test::TestInspector;
