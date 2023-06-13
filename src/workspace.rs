use crate::git::get_repository_info;
use crate::inspectors::{FileInspector, FileInspectorOptions};
use crate::models::PackageJsonFile;
use chrono::Utc;
use ignore::WalkBuilder;
use serde_json::{json, Map, Value};
use std::error::Error;
use std::path::{Path, PathBuf};

pub struct Workspace {
    pub working_dir: PathBuf,
    verbose: bool,
    inspectors: Vec<Box<dyn FileInspector>>,
}

impl Workspace {
    pub fn new(
        working_dir: PathBuf,
        inspectors: Vec<Box<dyn FileInspector>>,
        verbose: bool,
    ) -> Self {
        Workspace {
            working_dir,
            verbose,
            inspectors,
        }
    }

    /// Performs the workspace analysis using the registered file inspectors
    pub fn inspect(&mut self) -> Result<Value, Box<dyn Error>> {
        if self.verbose {
            println!("{}", self.working_dir.display());
        }

        let mut map = Map::new();

        map.insert(
            "report_date".to_owned(),
            Value::String(Utc::now().to_string()),
        );

        let modules: Vec<&str> = self
            .inspectors
            .iter()
            .map(|inspector| inspector.get_module_name())
            .collect();

        if let Some(project) = get_project_info(&self.working_dir, modules) {
            map.insert("project".to_owned(), project);
        }

        if let Some(repo) = get_repository_info(&self.working_dir) {
            map.insert("git".to_owned(), json!(repo));
        }

        self.run_inspectors(&mut map);
        Ok(Value::Object(map))
    }

    fn run_inspectors(&mut self, map: &mut Map<String, Value>) {
        for inspector in self.inspectors.iter_mut() {
            inspector.init(&self.working_dir, map);
        }

        for entry in WalkBuilder::new(&self.working_dir)
            .build()
            .filter_map(|entry| entry.ok())
        {
            // let f_name = entry.file_name().to_string_lossy();
            let entry_path = entry.path();
            let mut processed = false;

            let options = FileInspectorOptions {
                working_dir: self.working_dir.to_path_buf(),
                path: entry_path.to_path_buf(),
                relative_path: entry_path
                    .strip_prefix(&self.working_dir)
                    .unwrap()
                    .to_path_buf(),
            };

            for inspector in self.inspectors.iter_mut() {
                if entry_path.is_file() && inspector.supports_file(entry_path) {
                    inspector.inspect_file(&options, map);
                    processed = true;
                }
            }

            if self.verbose {
                println!(
                    "├── {} {}",
                    if processed { '✅' } else { '🔎' },
                    entry_path
                        .strip_prefix(&self.working_dir)
                        .unwrap()
                        .display()
                );
            }
        }

        for inspector in self.inspectors.iter_mut() {
            inspector.finalize(map);
        }
    }
}

fn get_project_info(working_dir: &Path, modules: Vec<&str>) -> Option<Value> {
    let package_json_path = working_dir.join("package.json");
    if package_json_path.exists() {
        let package = PackageJsonFile::from_file(&package_json_path).unwrap();

        return Some(json!({
            "name": package.name,
            "version": package.version,
            "modules": modules
        }));
    } else {
        println!("Warning: no package.json file found in the workspace");
    }

    None
}
