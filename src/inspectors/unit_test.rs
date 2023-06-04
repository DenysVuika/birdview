use super::utils;
use super::FileInspector;
use crate::workspace::Workspace;
use serde_json::{json, Map, Value};
use std::path::Path;

pub struct UnitTestInspector {
    total_files: i64,
    total_cases: i64,
}

impl UnitTestInspector {
    /// Creates a new instance of the inspector
    pub fn new() -> Self {
        UnitTestInspector {
            total_files: 0,
            total_cases: 0,
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

    fn inspect_file(
        &mut self,
        workspace: &Workspace,
        path: &Path,
        output: &mut Map<String, Value>,
    ) {
        let contents = std::fs::read_to_string(path).unwrap();
        let test_names = utils::extract_test_names(&contents);
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

        unit_tests.push(json!({
            "path": workspace_path,
            "cases": test_names,
        }));

        self.total_files += 1;
        self.total_cases += test_names.len() as i64;
    }

    fn finalize(&mut self, _workspace: &Workspace, output: &mut Map<String, Value>) {
        let stats = output
            .entry("stats")
            .or_insert(json!({}))
            .as_object_mut()
            .unwrap();

        stats.entry("tests").or_insert(json!({
            "unit_test": self.total_files,
            "unit_test_case": self.total_cases
        }));

        println!("Unit Tests");
        println!(" ├── Cases: {}", self.total_cases);
        println!(" └── Files: {}", self.total_files);
    }
}
