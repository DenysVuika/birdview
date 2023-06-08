use super::FileInspector;
use crate::inspectors::FileInspectorOptions;
use crate::models::PackageJsonFile;
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::path::Path;

#[derive(Serialize)]
pub struct PackageEntry {
    path: String,
    dependencies: Vec<DependencyEntry>,
}

#[derive(Serialize)]
pub struct DependencyEntry {
    name: String,
    version: String,
    dev: bool,
}

pub struct PackageJsonInspector {
    total_deps: i64,
    total_dev_deps: i64,
    packages: Vec<PackageEntry>,
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
    fn init(&mut self, _working_dir: &Path, _output: &mut Map<String, Value>) {}

    fn supports_file(&self, path: &Path) -> bool {
        path.is_file() && path.ends_with("package.json")
    }

    fn inspect_file(&mut self, options: &FileInspectorOptions, _output: &mut Map<String, Value>) {
        let package = PackageJsonFile::from_file(&options.path).unwrap();
        let mut dependencies: Vec<DependencyEntry> = Vec::new();

        if let Some(data) = package.dependencies {
            for (name, version) in data {
                dependencies.push(DependencyEntry {
                    name,
                    version,
                    dev: false,
                });
                self.total_deps += 1;
            }
        }

        if let Some(data) = package.dev_dependencies {
            for (name, version) in data {
                dependencies.push(DependencyEntry {
                    name,
                    version,
                    dev: true,
                });
                self.total_dev_deps += 1;
            }
        }

        let workspace_path = options.relative_path.display().to_string();

        self.packages.push(PackageEntry {
            path: workspace_path,
            dependencies,
        })
    }

    fn finalize(&mut self, output: &mut Map<String, Value>) {
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
    use crate::inspectors::utils::test_utils::options_from_file;
    use assert_fs::prelude::*;
    use assert_fs::NamedTempFile;
    use std::error::Error;

    #[test]
    fn supports_json_file() -> Result<(), Box<dyn Error>> {
        let file = NamedTempFile::new("package.json")?;
        file.touch()?;
        let inspector = PackageJsonInspector::new();
        assert!(inspector.supports_file(file.path()));

        file.close()?;
        Ok(())
    }

    #[test]
    fn requires_json_file_to_exist() {
        let path = Path::new("missing/package.json");
        let inspector = PackageJsonInspector::new();
        assert!(!inspector.supports_file(path));
    }

    #[test]
    fn writes_default_values_on_finalise() {
        let mut inspector: PackageJsonInspector = Default::default();

        let mut map: Map<String, Value> = Map::new();
        inspector.finalize(&mut map);

        assert_eq!(
            Value::Object(map),
            json!({
                "packages": [],
                "stats": {
                    "package": {
                        "files": 0,
                        "prod_deps": 0,
                        "dev_deps": 0
                    }
                }
            })
        );
    }

    #[test]
    fn writes_package_stats_on_finalise() {
        let mut inspector = PackageJsonInspector {
            total_deps: 20,
            total_dev_deps: 30,
            packages: vec![],
        };

        let mut map: Map<String, Value> = Map::new();
        inspector.finalize(&mut map);

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

    #[test]
    fn parses_package_dependencies() -> Result<(), Box<dyn Error>> {
        let file = NamedTempFile::new("package.json")?;
        file.write_str(
            r#"
            {
                "name": "test",
                "version": "1.0.0",
                "dependencies": {
                    "tslib": "^2.5.0"
                },
                "devDependencies": {
                    "@angular/cli": "14.1.3"
                }
            }
        "#,
        )?;

        let mut inspector = PackageJsonInspector::new();

        let mut map: Map<String, Value> = Map::new();
        let options = options_from_file(&file);

        inspector.inspect_file(&options, &mut map);
        inspector.finalize(&mut map);

        assert_eq!(
            Value::Object(map),
            json!({
                "packages": [
                    {
                        "path": "package.json",
                        "dependencies": [
                            {
                              "name": "tslib",
                              "version": "^2.5.0",
                              "dev": false
                            },
                            {
                              "name": "@angular/cli",
                              "version": "14.1.3",
                              "dev": true
                            }
                        ]
                    }
                ],
                "stats": {
                    "package": {
                        "files": 1,
                        "prod_deps": 1,
                        "dev_deps": 1
                    }
                }
            })
        );

        file.close()?;
        Ok(())
    }
}
