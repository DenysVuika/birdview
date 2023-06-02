use super::utils;
use super::FileInspector;
use crate::workspace::Workspace;
use serde_json::{json, Map, Value};
use std::path::Path;

#[derive(Default)]
pub struct UnitTestInspector {}

impl UnitTestInspector {
    /// Creates a new instance of the inspector
    pub fn new() -> Self {
        Default::default()
    }
}

impl FileInspector for UnitTestInspector {
    fn supports_file(&self, path: &Path) -> bool {
        path.is_file() && path.display().to_string().ends_with(".spec.ts")
    }

    fn inspect_file(&self, workspace: &Workspace, path: &Path, output: &mut Map<String, Value>) {
        let contents = std::fs::read_to_string(path).unwrap();
        let test_names: Vec<String> = utils::extract_test_names(&contents)
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let workspace_path = path
            .strip_prefix(&workspace.working_dir)
            .unwrap()
            .display()
            .to_string();

        let unit_tests = output
            .entry("unit_tests")
            .or_insert(json!([]))
            .as_array_mut()
            .unwrap();

        let entry = json!({
            "path": workspace_path,
            "cases": test_names,
        });

        unit_tests.push(entry);

        let total_files = output
            .entry("total_unit_test_files")
            .or_insert(json!(0))
            .as_i64()
            .unwrap();

        output["total_unit_test_files"] = json!(total_files + 1);

        let total_cases = output
            .entry("total_unit_test_cases")
            .or_insert(json!(0))
            .as_i64()
            .unwrap();

        output["total_unit_test_cases"] = json!(total_cases + test_names.len() as i64);
    }

    fn finalize(&self, _workspace: &Workspace, output: &mut Map<String, Value>) {
        let total_files = output
            .entry("total_unit_test_files")
            .or_insert(json!(0))
            .as_i64()
            .unwrap();

        let total_cases = output
            .entry("total_unit_test_cases")
            .or_insert(json!(0))
            .as_i64()
            .unwrap();

        println!(
            "unit test files (.spec.ts): {} ({} cases))",
            total_files, total_cases
        );
    }
}
