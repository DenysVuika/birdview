use super::utils;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

pub trait FileInspector {
    // Initialise inspector
    fn init(&mut self, working_dir: &Path, output: &mut Map<String, Value>);

    /// Check if the inspector supports the file
    fn supports_file(&self, path: &Path) -> bool;

    /// Run inspections for the file
    fn inspect_file(&mut self, options: &FileInspectorOptions, output: &mut Map<String, Value>);

    /// Perform final tasks after all inspectors finished
    fn finalize(&mut self, output: &mut Map<String, Value>);
}

pub struct FileInspectorOptions {
    pub working_dir: PathBuf,
    pub path: PathBuf,
    pub relative_path: PathBuf,
}

impl FileInspectorOptions {
    pub fn as_json(&self) -> Value {
        utils::load_json_file(&self.path)
    }

    pub fn read_content(&self) -> String {
        std::fs::read_to_string(&self.path).unwrap()
    }
}
