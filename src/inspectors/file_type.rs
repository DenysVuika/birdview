use super::FileInspector;
use crate::inspectors::FileInspectorOptions;
use rusqlite::Connection;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;
use uuid::Uuid;

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

    fn inspect_file(
        &mut self,
        connection: &Connection,
        project_id: &Uuid,
        options: &FileInspectorOptions,
        _output: &mut Map<String, Value>,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(ext) = options.relative_path.extension().and_then(OsStr::to_str) {
            let entry = self.types.entry(ext.to_owned()).or_insert(0);
            *entry += 1;
        }

        Ok(())
    }

    fn finalize(
        &mut self,
        connection: &Connection,
        project_id: &Uuid,
        output: &mut Map<String, Value>,
    ) -> Result<(), Box<dyn Error>> {
        if self.types.is_empty() {
            return Ok(());
        }

        output.entry("types").or_insert(json!(self.types));

        // println!("Project Files");
        // for (key, value) in &self.types {
        //     println!(" ├── {}: {}", key, value);
        // }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspectors::utils::test_utils::options_from_file;
    use assert_fs::prelude::*;
    use assert_fs::NamedTempFile;
    use std::error::Error;

    #[test]
    fn no_output_when_inspections_not_invoked() -> Result<(), Box<dyn Error>> {
        let conn = Connection::open_in_memory()?;
        let mut map: Map<String, Value> = Map::new();
        let mut inspector = FileTypeInspector::new();
        inspector.finalize(&conn, &Uuid::new_v4(), &mut map)?;

        assert_eq!(Value::Object(map), json!({}));
        Ok(())
    }

    #[test]
    fn should_parse_multiple_types() -> Result<(), Box<dyn Error>> {
        let conn = Connection::open_in_memory()?;
        let project_id = Uuid::new_v4();
        let file1 = NamedTempFile::new("test.spec.ts")?;
        file1.touch()?;

        let file2 = NamedTempFile::new("README.md")?;
        file2.touch()?;

        let mut map: Map<String, Value> = Map::new();
        let mut inspector = FileTypeInspector::new();

        inspector.inspect_file(&conn, &project_id, &options_from_file(&file1), &mut map)?;
        inspector.inspect_file(&conn, &project_id, &options_from_file(&file2), &mut map)?;
        inspector.finalize(&conn, &project_id, &mut map)?;

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
