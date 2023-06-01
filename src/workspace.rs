use crate::fs::{is_excluded, is_not_hidden};
use chrono::Utc;
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{json, Map, Value};
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

    pub fn inspect(&self) -> Value {
        let mut map = Map::new();

        let package_json_path = &self.working_dir.join("package.json");
        if package_json_path.exists() {
            println!("Found root package.json file");

            let file = File::open(package_json_path).unwrap();
            let reader = BufReader::new(file);
            let value: Value = serde_json::from_reader(reader).unwrap();

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
        Value::Object(map)
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
    }
}

pub trait FileInspector {
    fn supports_file(&self, path: &Path) -> bool;
    fn inspect_file(&self, workspace: &Workspace, path: &Path, output: &mut Map<String, Value>);
}

pub struct UnitTestInspector {}

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

        let v = serde_json::to_value(entry).unwrap();
        unit_tests.push(v);
    }
}

pub struct EndToEndTestInspector {}

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

        let v = serde_json::to_value(entry).unwrap();
        unit_tests.push(v);
    }
}

pub struct PackageJsonInspector {}

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

        if let Some(data) = value["dependencies"].as_object() {
            for (key, value) in data {
                let entry = json!({
                   "name": key,
                    "version": value,
                    "dev": false
                });
                dependencies.push(entry);
            }
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
        }

        let entry = json!({
            "path": workspace_path,
            "dependencies": dependencies
        });

        packages.push(entry);
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
