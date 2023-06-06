use super::FileInspector;
use crate::inspectors::FileInspectorOptions;
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{json, Map, Value};
use std::path::Path;

pub struct TestInspector {
    unit_test_cases: i64,
    unit_test_files: Vec<Value>,
    e2e_test_cases: i64,
    e2e_test_files: Vec<Value>,
}

impl TestInspector {
    /// Creates a new instance of the inspector
    pub fn new() -> Self {
        TestInspector {
            unit_test_cases: 0,
            unit_test_files: vec![],
            e2e_test_cases: 0,
            e2e_test_files: vec![],
        }
    }

    pub fn extract_test_names(contents: &str) -> Vec<&str> {
        // (\b(?:it|test)\b\(['"])(?P<name>.*?)(['"])
        // https://rustexp.lpil.uk/
        lazy_static! {
            static ref NAME_REGEX: Regex =
                Regex::new(r#"(\b(?:it|test)\b\(['"])(?P<name>.*?)(['"])"#).unwrap();
        }

        NAME_REGEX
            .captures_iter(contents)
            .map(|c| c.name("name").unwrap().as_str())
            .collect()
    }
}

impl Default for TestInspector {
    fn default() -> Self {
        TestInspector::new()
    }
}

impl FileInspector for TestInspector {
    fn supports_file(&self, path: &Path) -> bool {
        let display_path = path.display().to_string();
        path.is_file()
            && (display_path.ends_with(".spec.ts")
                || display_path.ends_with(".test.ts")
                || display_path.ends_with(".e2e.ts"))
    }

    fn inspect_file(&mut self, options: &FileInspectorOptions, _output: &mut Map<String, Value>) {
        let contents = options.read_content();
        let test_names = TestInspector::extract_test_names(&contents);

        if !test_names.is_empty() {
            let workspace_path = options.relative_path.display().to_string();
            let entry = json!({
                "path": workspace_path,
                "cases": test_names,
            });

            if workspace_path.ends_with(".spec.ts") {
                self.unit_test_files.push(entry);
                self.unit_test_cases += test_names.len() as i64;
            } else if workspace_path.ends_with(".test.ts") || workspace_path.ends_with(".e2e.ts") {
                self.e2e_test_files.push(entry);
                self.e2e_test_cases += test_names.len() as i64;
            }
        }
    }

    fn finalize(&mut self, output: &mut Map<String, Value>) {
        output
            .entry("unit_tests")
            .or_insert(json!(self.unit_test_files));

        output
            .entry("e2e_tests")
            .or_insert(json!(self.e2e_test_files));

        let stats = output
            .entry("stats")
            .or_insert(json!({}))
            .as_object_mut()
            .unwrap();

        stats.entry("tests").or_insert(json!({
            "unit_test": self.unit_test_files.len(),
            "unit_test_case": self.unit_test_cases,
            "e2e_test": self.e2e_test_files.len(),
            "e2e_test_case": self.e2e_test_cases
        }));

        if !self.unit_test_files.is_empty() {
            println!("Unit Tests");
            println!(" ├── Cases: {}", self.unit_test_cases);
            println!(" └── Files: {}", self.unit_test_files.len());
        }

        if !self.e2e_test_files.is_empty() {
            println!("E2E Tests");
            println!(" ├── Cases: {}", self.e2e_test_cases);
            println!(" └── Files: {}", self.e2e_test_files.len());
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
    fn extracts_single_test_name() {
        let input = "it('should reset selected nodes from store', () => {";
        assert_eq!(
            vec!["should reset selected nodes from store"],
            TestInspector::extract_test_names(input)
        );
    }

    #[test]
    fn extracts_multiple_test_names() {
        let input = "\
            it('should reset selected nodes from store', () => {\
            it('should return false when entry is not shared', () => {
        ";
        assert_eq!(
            vec![
                "should reset selected nodes from store",
                "should return false when entry is not shared"
            ],
            TestInspector::extract_test_names(input)
        );
    }

    #[test]
    fn extracts_playwright_test_names() {
        let input = "test('Create a rule with condition', async ({ personalFiles, nodesPage })";
        assert_eq!(
            vec!["Create a rule with condition"],
            TestInspector::extract_test_names(input)
        );
    }

    #[test]
    fn requires_spec_file_to_exist() {
        let path = Path::new("missing/test.spec.ts");
        let inspector = TestInspector::new();
        assert!(!inspector.supports_file(path));
    }

    #[test]
    fn supports_spec_file() -> Result<(), Box<dyn std::error::Error>> {
        let file = NamedTempFile::new("test.spec.ts")?;
        file.touch()?;
        let inspector: TestInspector = Default::default();
        assert!(inspector.supports_file(file.path()));

        file.close()?;
        Ok(())
    }

    #[test]
    fn writes_default_values_on_finalise() {
        let mut inspector: TestInspector = Default::default();

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

        let mut inspector = TestInspector::new();

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
