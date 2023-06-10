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
    fn get_module_name(&self) -> &str {
        "file-types"
    }

    fn init(&mut self, _working_dir: &Path, _output: &mut Map<String, Value>) {}

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
        if self.types.is_empty() {
            return;
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspectors::utils::test_utils::options_from_file;
    use assert_fs::prelude::*;
    use assert_fs::NamedTempFile;

    #[test]
    fn no_output_when_inspections_not_invoked() {
        let mut map: Map<String, Value> = Map::new();
        let mut inspector = FileTypeInspector::new();
        inspector.finalize(&mut map);

        assert_eq!(Value::Object(map), json!({}));
    }

    #[test]
    fn should_parse_multiple_types() -> Result<(), Box<dyn std::error::Error>> {
        let file1 = NamedTempFile::new("test.spec.ts")?;
        file1.touch()?;

        let file2 = NamedTempFile::new("README.md")?;
        file2.touch()?;

        let mut map: Map<String, Value> = Map::new();
        let mut inspector = FileTypeInspector::new();

        inspector.inspect_file(&options_from_file(&file1), &mut map);
        inspector.inspect_file(&options_from_file(&file2), &mut map);
        inspector.finalize(&mut map);

        assert_eq!(
            Value::Object(map),
            json!({
                "stats": {
                    "types": {
                        "ts": 1,
                        "md": 1
                    }
                }
            })
        );

        file1.close()?;
        file2.close()?;
        Ok(())
    }
}
