use super::FileInspector;
use crate::inspectors::FileInspectorOptions;
use crate::workspace::Workspace;
use serde_json::{json, Map, Value};
use std::path::Path;

pub struct PackageJsonInspector {
    total_deps: i64,
    total_dev_deps: i64,
    packages: Vec<Value>,
}

impl PackageJsonInspector {
    /// Creates a new instance of the inspector
    pub fn new() -> Self {
        PackageJsonInspector {
            total_deps: 0,
            total_dev_deps: 0,
            packages: vec![],
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

    fn inspect_file(&mut self, options: &FileInspectorOptions, _output: &mut Map<String, Value>) {
        let value = options.as_json();
        let mut dependencies: Vec<Value> = Vec::new();

        if let Some(data) = value["dependencies"].as_object() {
            for (key, value) in data {
                dependencies.push(json!({
                    "name": key,
                    "version": value,
                    "dev": false
                }));
            }
            self.total_deps += data.len() as i64;
        }

        if let Some(data) = value["devDependencies"].as_object() {
            for (key, value) in data {
                dependencies.push(json!({
                    "name": key,
                    "version": value,
                    "dev": true
                }));
            }
            self.total_dev_deps += data.len() as i64;
        }

        let workspace_path = options.relative_path.display().to_string();

        self.packages.push(json!({
            "path": workspace_path,
            "dependencies": dependencies
        }));
    }

    fn finalize(&mut self, _workspace: &Workspace, output: &mut Map<String, Value>) {
        output.entry("packages").or_insert(json!(self.packages));

        let stats = output
            .entry("stats")
            .or_insert(json!({}))
            .as_object_mut()
            .unwrap();

        stats.entry("package").or_insert(json!({
            "files": self.packages.len(),
            "prod_deps": self.total_deps,
            "dev_deps": self.total_dev_deps
        }));

        println!("Packages");
        println!(" ├── Files: {}", self.packages.len());
        println!(" ├── Dependencies: {}", self.total_deps);
        println!(" └── Dev dependencies: {}", self.total_dev_deps);

        // cleanup
        self.packages = vec![];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn writes_package_stats_on_finalise() {
        let mut inspector = PackageJsonInspector {
            total_deps: 20,
            total_dev_deps: 30,
            packages: vec![],
        };

        let workspace = Workspace::setup(PathBuf::from("."), false);
        let mut map: Map<String, Value> = Map::new();
        inspector.finalize(&workspace, &mut map);

        assert_eq!(
            Value::Object(map),
            json!({
                "packages": [],
                "stats": {
                    "package": {
                        "files": 0,
                        "prod_deps": 20,
                        "dev_deps": 30
                    }
                }
            })
        );
    }
}
