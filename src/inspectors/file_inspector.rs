use serde_json::{Map, Value};
use std::path::Path;

use crate::workspace::Workspace;

pub trait FileInspector {
    /// Check if the inspector supports the file
    fn supports_file(&self, path: &Path) -> bool;

    /// Run inspections for the file
    fn inspect_file(&mut self, workspace: &Workspace, path: &Path, output: &mut Map<String, Value>);

    /// Perform final tasks after all inspectors finished
    fn finalize(&mut self, workspace: &Workspace, output: &mut Map<String, Value>);
}
