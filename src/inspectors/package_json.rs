use super::utils;
use super::FileInspector;
use crate::workspace::Workspace;
use serde_json::{json, Map, Value};
use std::path::Path;

pub struct PackageJsonInspector {
    total_package_files: i64,
    total_deps: i64,
    total_dev_deps: i64,
}

impl PackageJsonInspector {
    /// Creates a new instance of the inspector
    pub fn new() -> Self {
        PackageJsonInspector {
            total_package_files: 0,
            total_deps: 0,
            total_dev_deps: 0,
        }
    }
}

impl Default for PackageJsonInspector {
    fn default() -> Self {
        PackageJsonInspector::new()
    }
}

impl FileInspector for PackageJsonInspector {
    fn supports_file(&self, path: &Path) -> bool {
        path.is_file() && path.ends_with("package.json")
    }

    fn inspect_file(
        &mut self,
        workspace: &Workspace,
        path: &Path,
        output: &mut Map<String, Value>,
    ) {
        let value = utils::load_json_file(path);

        let packages = output
            .entry("packages")
            .or_insert(json!([]))
            .as_array_mut()
            .unwrap();

        let workspace_path = path
            .strip_prefix(&workspace.working_dir)
            .unwrap()
            .display()
            .to_string();

        let mut dependencies: Vec<Value> = Vec::new();

        if let Some(data) = value["dependencies"].as_object() {
            for (key, value) in data {
                let entry = json!({
                   "name": key,
                    "version": value,
                    "dev": false
                });
                dependencies.push(entry);
            }
            self.total_deps += data.len() as i64;
        }

        if let Some(data) = value["devDependencies"].as_object() {
            for (key, value) in data {
                let entry = json!({
                   "name": key,
                    "version": value,
                    "dev": true
                });
                dependencies.push(entry);
            }
            self.total_dev_deps += data.len() as i64;
        }

        let entry = json!({
            "path": workspace_path,
            "dependencies": dependencies
        });

        packages.push(entry);
        self.total_package_files += 1;
    }

    fn finalize(&mut self, _workspace: &Workspace, output: &mut Map<String, Value>) {
        let stats = output
            .entry("stats")
            .or_insert(json!({}))
            .as_object_mut()
            .unwrap();

        stats
            .entry("package")
            .or_insert(json!(self.total_package_files));
        stats
            .entry("package_deps")
            .or_insert(json!(self.total_deps));
        stats
            .entry("package_dev_deps")
            .or_insert(json!(self.total_dev_deps));

        println!("Packages");
        println!(" ├── Files: {}", self.total_package_files);
        println!(" ├── Dependencies: {}", self.total_deps);
        println!(" └── Dev dependencies: {}", self.total_dev_deps);
    }
}
