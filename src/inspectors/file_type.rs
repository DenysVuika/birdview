use super::FileInspector;
use crate::inspectors::FileInspectorOptions;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;

pub struct FileTypeInspector {
    types: HashMap<String, i64>,
}

impl FileTypeInspector {
    pub fn new() -> Self {
        FileTypeInspector {
            types: HashMap::new(),
        }
    }
}

impl Default for FileTypeInspector {
    fn default() -> Self {
        FileTypeInspector::new()
    }
}

impl FileInspector for FileTypeInspector {
    fn supports_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn inspect_file(&mut self, options: &FileInspectorOptions, _output: &mut Map<String, Value>) {
        if let Some(ext) = options.relative_path.extension().and_then(OsStr::to_str) {
            let entry = self.types.entry(ext.to_owned()).or_insert(0);
            *entry += 1;
        }
    }

    fn finalize(&mut self, output: &mut Map<String, Value>) {
        let stats = output
            .entry("stats")
            .or_insert(json!({}))
            .as_object_mut()
            .unwrap();

        stats.entry("types").or_insert(json!(self.types));

        println!("Project Files");
        for (key, value) in &self.types {
            println!(" ├── {}: {}", key, value);
        }
    }
}
