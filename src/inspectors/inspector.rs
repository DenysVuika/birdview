use rusqlite::Connection;
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
    ) -> Result<(), Box<dyn Error>>;
}

pub struct FileInspectorOptions {
    pub working_dir: PathBuf,
    pub path: PathBuf,
    pub relative_path: PathBuf,
}

impl FileInspectorOptions {
    pub fn read_content(&self) -> String {
        std::fs::read_to_string(&self.path).unwrap()
    }
}
