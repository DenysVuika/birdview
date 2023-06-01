use crate::fs::{is_excluded, is_not_hidden};
use chrono::Utc;
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{json, Map, Value};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct Workspace {
    working_dir: PathBuf,
    file_inspectors: Vec<Box<dyn FileInspector>>,
}

impl Workspace {
    pub fn setup(working_dir: PathBuf, inspectors: Vec<Box<dyn FileInspector>>) -> Workspace {
        Workspace {
            working_dir,
            file_inspectors: inspectors,
        }
    }

    pub fn load_json_file(path: &Path) -> Value {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let value: Value = serde_json::from_reader(reader).unwrap();
        value
    }

    pub fn inspect(&self) -> Result<Value, Box<dyn Error>> {
        let mut map = Map::new();

        let package_json_path = &self.working_dir.join("package.json");
        if package_json_path.exists() {
            let file = File::open(package_json_path)?;
            let reader = BufReader::new(file);
            let value: Value = serde_json::from_reader(reader)?;

            map.insert("project_name".to_owned(), value["name"].to_owned());
            map.insert("project_version".to_owned(), value["version"].to_owned());
        } else {
            println!("Warning: no package.json file found in the workspace");
        }

        map.insert(
            "report_date".to_owned(),
            Value::String(Utc::now().to_string()),
        );

        self.run_inspectors(&mut map);
        Ok(Value::Object(map))
    }

    fn run_inspectors(&self, map: &mut Map<String, Value>) {
        let walker = WalkDir::new(&self.working_dir)
            .follow_links(true)
            .into_iter();

        for entry in walker
            .filter_entry(|e| is_not_hidden(e) && !is_excluded(e))
            .filter_map(|e| e.ok())
        {
            // let f_name = entry.file_name().to_string_lossy();
            let entry_path = entry.path();

            for inspector in &self.file_inspectors {
                if inspector.supports_file(&entry_path) {
                    inspector.inspect_file(&self, entry_path, map);
                }
            }
        }

        for inspector in &self.file_inspectors {
            inspector.finalize(&self, map);
        }
    }
}

pub trait FileInspector {
    /// Check if the inspector supports the file
    fn supports_file(&self, path: &Path) -> bool;
    /// Run inspections for the file
    fn inspect_file(&self, workspace: &Workspace, path: &Path, output: &mut Map<String, Value>);
    /// Perform final tasks after all inspectors finished
    fn finalize(&self, workspace: &Workspace, output: &mut Map<String, Value>);
}

pub struct UnitTestInspector {}

impl UnitTestInspector {
    /// Creates a new instance of the inspector
    pub fn new() -> UnitTestInspector {
        UnitTestInspector {}
    }
}

impl FileInspector for UnitTestInspector {
    fn supports_file(&self, path: &Path) -> bool {
        path.is_file() && path.display().to_string().ends_with(".spec.ts")
    }

    fn inspect_file(&self, workspace: &Workspace, path: &Path, output: &mut Map<String, Value>) {
        let contents = std::fs::read_to_string(path).unwrap();
        let test_names: Vec<String> = extract_test_names(&contents)
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

pub struct EndToEndTestInspector {}

impl EndToEndTestInspector {
    /// Creates a new instance of the inspector
    pub fn new() -> EndToEndTestInspector {
        EndToEndTestInspector {}
    }
}

impl FileInspector for EndToEndTestInspector {
    fn supports_file(&self, path: &Path) -> bool {
        path.is_file()
            && (path.display().to_string().ends_with(".test.ts")
                || path.display().to_string().ends_with(".e2e.ts"))
    }

    fn inspect_file(&self, workspace: &Workspace, path: &Path, output: &mut Map<String, Value>) {
        let contents = std::fs::read_to_string(path).unwrap();
        let test_names: Vec<String> = extract_test_names(&contents)
            .into_iter()
            .map(|s| s.to_string())
            .collect();
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

        let total_files = output
            .entry("total_e2e_test_files")
            .or_insert(json!(0))
            .as_i64()
            .unwrap();
        output["total_e2e_test_files"] = json!(total_files + 1);

        let total_cases = output
            .entry("total_e2e_test_cases")
            .or_insert(json!(0))
            .as_i64()
            .unwrap();
        output["total_e2e_test_cases"] = json!(total_cases + test_names.len() as i64);
    }

    fn finalize(&self, _workspace: &Workspace, output: &mut Map<String, Value>) {
        let total_files = output
            .entry("total_e2e_test_files")
            .or_insert(json!(0))
            .as_i64()
            .unwrap();
        let total_cases = output
            .entry("total_e2e_test_cases")
            .or_insert(json!(0))
            .as_i64()
            .unwrap();

        println!(
            "e2e test files (.test.ts, .e2e.ts): {} ({} cases))",
            total_files, total_cases
        );
    }
}

pub struct PackageJsonInspector {}

impl PackageJsonInspector {
    /// Creates a new instance of the inspector
    pub fn new() -> PackageJsonInspector {
        PackageJsonInspector {}
    }
}

impl FileInspector for PackageJsonInspector {
    fn supports_file(&self, path: &Path) -> bool {
        path.is_file() && path.ends_with("package.json")
    }

    fn inspect_file(&self, workspace: &Workspace, path: &Path, output: &mut Map<String, Value>) {
        let value = Workspace::load_json_file(path);

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

fn extract_test_names(contents: &str) -> Vec<&str> {
    // (\b(?:it|test)\b\(['"])(?P<name>.*?)(['"])
    // https://rustexp.lpil.uk/
    lazy_static! {
        static ref NAME_REGEX: Regex =
            Regex::new(r#"(\b(?:it|test)\b\(['"])(?P<name>.*?)(['"])"#).unwrap();
    }

    NAME_REGEX
        .captures_iter(&contents)
        .map(|c| c.name("name").unwrap().as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_single_test_name() {
        let input = "it('should reset selected nodes from store', () => {";
        assert_eq!(
            vec!["should reset selected nodes from store"],
            extract_test_names(input)
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
            extract_test_names(input)
        );
    }

    #[test]
    fn extracts_playwright_test_names() {
        let input = "test('Create a rule with condition', async ({ personalFiles, nodesPage })";
        assert_eq!(
            vec!["Create a rule with condition"],
            extract_test_names(input)
        );
    }
}
