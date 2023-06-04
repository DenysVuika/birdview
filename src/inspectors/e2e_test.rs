use super::utils;
use super::FileInspector;
use crate::workspace::Workspace;
use serde_json::{json, Map, Value};
use std::path::Path;

pub struct EndToEndTestInspector {
    total_files: i64,
    total_cases: i64,
}

impl EndToEndTestInspector {
    /// Creates a new instance of the inspector
    pub fn new() -> Self {
        EndToEndTestInspector {
            total_files: 0,
            total_cases: 0,
        }
    }
}

impl Default for EndToEndTestInspector {
    fn default() -> Self {
        EndToEndTestInspector::new()
    }
}

impl FileInspector for EndToEndTestInspector {
    fn supports_file(&self, path: &Path) -> bool {
        let display_path = path.display().to_string();
        path.is_file() && (display_path.ends_with(".test.ts") || display_path.ends_with(".e2e.ts"))
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
            .entry("e2e_tests")
            .or_insert(json!([]))
            .as_array_mut()
            .unwrap();

        let entry = json!({
            "path": workspace_path,
            "cases": test_names,
        });

        unit_tests.push(entry);

        self.total_files += 1;
        self.total_cases += test_names.len() as i64;
    }

    fn finalize(&mut self, _workspace: &Workspace, output: &mut Map<String, Value>) {
        let stats = output
            .entry("stats")
            .or_insert(json!({}))
            .as_object_mut()
            .unwrap();

        let tests = stats
            .entry("tests")
            .or_insert(json!({}))
            .as_object_mut()
            .unwrap();

        tests.entry("e2e_test").or_insert(json!(self.total_files));
        tests
            .entry("e2e_test_case")
            .or_insert(json!(self.total_cases));

        println!("E2E Tests");
        println!(" ├── Cases: {}", self.total_cases);
        println!(" └── Files: {}", self.total_files);
    }
}
