use super::utils;
use super::FileInspector;
use crate::inspectors::FileInspectorOptions;
use serde_json::{json, Map, Value};
use std::path::Path;

pub struct UnitTestInspector {
    total_cases: i64,
    test_files: Vec<Value>,
}

impl UnitTestInspector {
    /// Creates a new instance of the inspector
    pub fn new() -> Self {
        UnitTestInspector {
            total_cases: 0,
            test_files: vec![],
        }
    }
}

impl Default for UnitTestInspector {
    fn default() -> Self {
        UnitTestInspector::new()
    }
}

impl FileInspector for UnitTestInspector {
    fn supports_file(&self, path: &Path) -> bool {
        path.is_file() && path.display().to_string().ends_with(".spec.ts")
    }

    fn inspect_file(&mut self, options: &FileInspectorOptions, _output: &mut Map<String, Value>) {
        let contents = options.read_content();
        let test_names = utils::extract_test_names(&contents);

        if !test_names.is_empty() {
            let workspace_path = options.relative_path.display().to_string();

            self.test_files.push(json!({
                "path": workspace_path,
                "cases": test_names,
            }));

            self.total_cases += test_names.len() as i64;
        }
    }

    fn finalize(&mut self, output: &mut Map<String, Value>) {
        output.entry("unit_tests").or_insert(json!(self.test_files));

        let stats = output
            .entry("stats")
            .or_insert(json!({}))
            .as_object_mut()
            .unwrap();

        stats.entry("tests").or_insert(json!({
            "unit_test": self.test_files.len(),
            "unit_test_case": self.total_cases
        }));

        println!("Unit Tests");
        println!(" ├── Cases: {}", self.total_cases);
        println!(" └── Files: {}", self.test_files.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspectors::utils::test_utils::options_from_file;
    use assert_fs::prelude::*;
    use assert_fs::NamedTempFile;

    #[test]
    fn requires_spec_file_to_exist() {
        let path = Path::new("missing/test.spec.ts");
        let inspector = UnitTestInspector::new();
        assert!(!inspector.supports_file(path));
    }

    #[test]
    fn supports_spec_file() -> Result<(), Box<dyn std::error::Error>> {
        let file = NamedTempFile::new("test.spec.ts")?;
        file.touch()?;
        let inspector: UnitTestInspector = Default::default();
        assert!(inspector.supports_file(file.path()));

        file.close()?;
        Ok(())
    }

    #[test]
    fn writes_default_values_on_finalise() {
        let mut inspector: UnitTestInspector = Default::default();

        let mut map: Map<String, Value> = Map::new();
        inspector.finalize(&mut map);

        assert_eq!(
            Value::Object(map),
            json!({
                "unit_tests": [],
                "stats": {
                    "tests": {
                        "unit_test": 0,
                        "unit_test_case": 0
                    }
                }
            })
        );
    }

    #[test]
    fn parses_unit_tests() -> Result<(), Box<dyn std::error::Error>> {
        let file = NamedTempFile::new("tests.spec.ts")?;
        let content = r#"
            describe('test suite', () => {
                it('should have default LTR direction value', () => {});
                it('should change direction on textOrientation event', () => {});
            });
        "#;
        file.write_str(content)?;

        let mut inspector = UnitTestInspector::new();

        let mut map: Map<String, Value> = Map::new();
        let options = options_from_file(&file);

        inspector.inspect_file(&options, &mut map);
        inspector.finalize(&mut map);

        assert_eq!(
            Value::Object(map),
            json!({
                "unit_tests": [
                    {
                        "path": "tests.spec.ts",
                        "cases": [
                            "should have default LTR direction value",
                            "should change direction on textOrientation event"
                        ]
                    }
                ],
                "stats": {
                    "tests": {
                        "unit_test": 1,
                        "unit_test_case": 2
                    }
                }
            })
        );

        file.close()?;
        Ok(())
    }
}
