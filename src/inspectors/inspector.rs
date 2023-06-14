use super::utils;
use rusqlite::Connection;
use serde_json::{Map, Value};
use std::error::Error;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub trait FileInspector {
    /// Check if the inspector supports the file
    fn supports_file(&self, path: &Path) -> bool;

    /// Run inspections for the file
    fn inspect_file(
        &mut self,
        connection: &Connection,
        project_id: &Uuid,
        options: &FileInspectorOptions,
        output: &mut Map<String, Value>,
    ) -> Result<(), Box<dyn Error>>;

    /// Perform final tasks after all inspectors finished
    fn finalize(
        &mut self,
        connection: &Connection,
        project_id: &Uuid,
        output: &mut Map<String, Value>,
    ) -> Result<(), Box<dyn Error>>;
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
