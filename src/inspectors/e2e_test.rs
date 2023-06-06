use super::utils;
use super::FileInspector;
use crate::inspectors::FileInspectorOptions;
use serde_json::{json, Map, Value};
use std::path::Path;

pub struct EndToEndTestInspector {
    total_cases: i64,
    test_files: Vec<Value>,
}

impl EndToEndTestInspector {
    /// Creates a new instance of the inspector
    pub fn new() -> Self {
        EndToEndTestInspector {
            total_cases: 0,
            test_files: vec![],
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
        output.entry("e2e_tests").or_insert(json!(self.test_files));

        let stats = output
            .entry("stats")
            .or_insert(json!({}))
            .as_object_mut()
            .unwrap();

        stats.entry("tests").or_insert(json!({
            "e2e_test": self.test_files.len(),
            "e2e_test_case": self.total_cases
        }));

        println!("E2E Tests");
        println!(" ├── Cases: {}", self.total_cases);
        println!(" └── Files: {}", self.test_files.len());
    }
}
