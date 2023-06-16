use super::FileInspector;
use crate::inspectors::FileInspectorOptions;
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use rusqlite::{params, Connection};
use std::path::Path;
use uuid::Uuid;

pub struct TestInspector {}

impl TestInspector {
    /// Creates a new instance of the inspector
    pub fn new() -> Self {
        TestInspector {}
    }

    pub fn extract_test_names(contents: &str) -> Vec<String> {
        // (\b(?:it|test)\b\(['"])(?P<name>.*?)(['"])
        // https://rustexp.lpil.uk/
        lazy_static! {
            static ref NAME_REGEX: Regex =
                Regex::new(r#"(\b(?:it|test)\b\(['"])(?P<name>.*?)(['"])"#).unwrap();
        }

        NAME_REGEX
            .captures_iter(contents)
            .map(|c| c.name("name").unwrap().as_str().to_owned())
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

    fn inspect_file(&self, conn: &Connection, opts: &FileInspectorOptions) -> Result<()> {
        let contents = opts.read_content();
        let test_names = TestInspector::extract_test_names(&contents);

        if !test_names.is_empty() {
            let test_id = Uuid::new_v4();
            let workspace_path = &opts.relative_path;
            let project_id = &opts.project_id;

            if workspace_path.ends_with(".spec.ts") {
                conn.execute(
                    "INSERT INTO unit_tests (id, project_id, path) VALUES (?1, ?2, ?3)",
                    params![test_id, project_id, workspace_path],
                )?;

                let mut stmt =
                    conn.prepare("INSERT INTO test_cases (test_id, name) VALUES (?1, ?2)")?;

                // todo: slow
                for name in test_names {
                    stmt.execute(params![test_id, name])?;
                }
            } else if workspace_path.ends_with(".test.ts") || workspace_path.ends_with(".e2e.ts") {
                conn.execute(
                    "INSERT INTO e2e_tests (id, project_id, path) VALUES (?1, ?2, ?3)",
                    params![test_id, project_id, workspace_path],
                )?;

                let mut stmt =
                    conn.prepare("INSERT INTO test_cases (test_id, name) VALUES (?1, ?2)")?;

                // todo: slow
                for name in test_names {
                    stmt.execute(params![test_id, name])?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use assert_fs::NamedTempFile;
    use std::error::Error;

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
    fn supports_spec_file() -> Result<(), Box<dyn Error>> {
        let file = NamedTempFile::new("test.spec.ts")?;
        file.touch()?;
        let inspector: TestInspector = Default::default();
        assert!(inspector.supports_file(file.path()));

        file.close()?;
        Ok(())
    }

    /*
    fn parses_unit_tests() -> Result<(), Box<dyn Error>> {
        let conn = Connection::open_in_memory()?;
        let project_id = Uuid::new_v4();
        let file = NamedTempFile::new("tests.e2e.ts")?;
        let content = r#"
            describe('test suite', () => {
                it('should have default LTR direction value', () => {});
                it('should change direction on textOrientation event', () => {});
            });
        "#;
        file.write_str(content)?;

        let mut inspector = TestInspector::new();

        inspector.inspect_file(&conn, &project_id, &options_from_file(&file))?;
        // inspector.finalize(&conn, &project_id, &mut map)?;

        assert_eq!(
            Value::Object(map),
            json!({
                "unit_tests": [],
                "e2e_tests": [
                    {
                        "path": "tests.e2e.ts",
                        "cases": [
                            "should have default LTR direction value",
                            "should change direction on textOrientation event"
                        ]
                    }
                ],
                "stats": {
                    "tests": {
                        "unit_test": 0,
                        "unit_test_case": 0,
                        "e2e_test": 1,
                        "e2e_test_case": 2
                    }
                }
            })
        );

        file.close()?;
        Ok(())
    }
    */

    /*
    fn parses_e2e_tests() -> Result<(), Box<dyn Error>> {
        let conn = Connection::open_in_memory()?;
        let project_id = Uuid::new_v4();

        let file = NamedTempFile::new("tests.spec.ts")?;
        let content = r#"
            describe('test suite', () => {
                it('should have default LTR direction value', () => {});
                it('should change direction on textOrientation event', () => {});
            });
        "#;
        file.write_str(content)?;

        let mut inspector = TestInspector::new();

        let options = options_from_file(&file);

        inspector.inspect_file(&conn, &project_id, &options)?;
        // inspector.finalize(&conn, &Uuid::new_v4(), &mut map)?;

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
                "e2e_tests": [],
                "stats": {
                    "tests": {
                        "unit_test": 1,
                        "unit_test_case": 2,
                        "e2e_test": 0,
                        "e2e_test_case": 0
                    }
                }
            })
        );

        file.close()?;
        Ok(())
    }
    */
}
