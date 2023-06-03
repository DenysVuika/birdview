use super::FileInspector;
use crate::workspace::Workspace;
use serde_json::{json, Map, Value};
use std::path::Path;

pub struct MarkdownInspector {
    total_files: i64,
}

impl MarkdownInspector {
    pub fn new() -> Self {
        MarkdownInspector { total_files: 0 }
    }
}

impl Default for MarkdownInspector {
    fn default() -> Self {
        MarkdownInspector::new()
    }
}

impl FileInspector for MarkdownInspector {
    fn supports_file(&self, path: &Path) -> bool {
        path.is_file() && path.display().to_string().ends_with(".md")
    }

    fn inspect_file(
        &mut self,
        _workspace: &Workspace,
        _path: &Path,
        _output: &mut Map<String, Value>,
    ) {
        self.total_files += 1;
    }

    fn finalize(&mut self, _workspace: &Workspace, output: &mut Map<String, Value>) {
        let stats = output
            .entry("stats")
            .or_insert(json!({}))
            .as_object_mut()
            .unwrap();

        stats.entry("markdown").or_insert(json!(self.total_files));

        println!("Markdown files: {}", self.total_files);
    }
}
