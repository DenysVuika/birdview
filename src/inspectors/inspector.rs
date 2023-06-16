use anyhow::Result;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub trait FileInspector {
    /// Check if the inspector supports the file
    fn supports_file(&self, path: &Path) -> bool;

    /// Run inspections for the file
    fn inspect_file(&self, conn: &Connection, opts: &FileInspectorOptions) -> Result<()>;
}

pub struct FileInspectorOptions {
    pub project_id: Uuid,
    pub path: PathBuf,
    pub relative_path: String,
    pub url: Option<String>,
}

impl FileInspectorOptions {
    pub fn read_content(&self) -> String {
        std::fs::read_to_string(&self.path).unwrap()
    }
}
