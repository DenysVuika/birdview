use super::utils;
use super::FileInspector;
use crate::workspace::Workspace;
use serde_json::{json, Map, Value};
use std::path::Path;

#[derive(Default)]
pub struct PackageJsonInspector {}

impl PackageJsonInspector {
    /// Creates a new instance of the inspector
    pub fn new() -> Self {
        Default::default()
    }
}

impl FileInspector for PackageJsonInspector {
    fn supports_file(&self, path: &Path) -> bool {
        path.is_file() && path.ends_with("package.json")
    }

    fn inspect_file(&self, workspace: &Workspace, path: &Path, output: &mut Map<String, Value>) {
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
        let mut total_deps: i64 = 0;
        let mut total_dev_deps: i64 = 0;

        if let Some(data) = value["dependencies"].as_object() {
            for (key, value) in data {
                let entry = json!({
                   "name": key,
                    "version": value,
                    "dev": false
                });
                dependencies.push(entry);
            }
            total_deps = data.len() as i64;
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
            total_dev_deps = data.len() as i64;
        }

        let entry = json!({
            "path": workspace_path,
            "dependencies": dependencies
        });

        packages.push(entry);

        let total_package_files = output
            .entry("total_package_files")
            .or_insert(json!(0))
            .as_i64()
            .unwrap();
        output["total_package_files"] = json!(total_package_files + 1);

        let total_package_deps = output
            .entry("total_package_deps")
            .or_insert(json!(0))
            .as_i64()
            .unwrap();
        output["total_package_deps"] = json!(total_package_deps + total_deps);

        let total_package_dev_deps = output
            .entry("total_package_dev_deps")
            .or_insert(json!(0))
            .as_i64()
            .unwrap();
        output["total_package_dev_deps"] = json!(total_package_dev_deps + total_dev_deps);
    }

    fn finalize(&self, _workspace: &Workspace, output: &mut Map<String, Value>) {
        let total_package_files = output
            .entry("total_package_files")
            .or_insert(json!(0))
            .as_i64()
            .unwrap();

        let total_package_deps = output
            .entry("total_package_deps")
            .or_insert(json!(0))
            .as_i64()
            .unwrap();

        let total_package_dev_deps = output
            .entry("total_package_dev_deps")
            .or_insert(json!(0))
            .as_i64()
            .unwrap();

        println!(
            "package.json files: {} ({} deps, {} dev deps)",
            total_package_files, total_package_deps, total_package_dev_deps
        );
    }
}
