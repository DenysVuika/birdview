use crate::inspectors::{FileInspector, FileInspectorOptions};
use crate::models::PackageJsonFile;
use chrono::Utc;
use git2::Repository;
use serde_json::{json, Map, Value};
use std::error::Error;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

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

        let project = map
            .entry("project")
            .or_insert(json!({
                "name": "unknown",
                "version": "unknown",
                "modules": modules,
            }))
            .as_object_mut()
            .unwrap();

        let package_json_path = &self.working_dir.join("package.json");
        if package_json_path.exists() {
            if self.verbose {
                println!(
                    "├── {}",
                    package_json_path.strip_prefix(&self.working_dir)?.display()
                );
            }
            let package = PackageJsonFile::from_file(package_json_path)?;

            project["name"] = Value::String(package.name.unwrap());
            project["version"] = Value::String(package.version.unwrap());
        } else {
            println!("Warning: no package.json file found in the workspace");
        }

        match Repository::open(&self.working_dir) {
            Err(..) => println!("Git repository not found"),
            Ok(repo) => {
                let remote = repo.find_remote("origin")?;
                let remote_url = remote.url().unwrap().strip_suffix(".git").unwrap();

                project.entry("git").or_insert(json!({
                    "remote": remote_url,
                    "branch": repo.head()?.shorthand().unwrap(),
                    "target": repo.head()?.target().unwrap().to_string(),
                }));
            }
        }

        self.run_inspectors(&mut map);
        Ok(Value::Object(map))
    }

    fn run_inspectors(&mut self, map: &mut Map<String, Value>) {
        for inspector in self.inspectors.iter_mut() {
            inspector.init(&self.working_dir, map);
        }

        let walker = WalkDir::new(&self.working_dir)
            .follow_links(true)
            .into_iter();

        for entry in walker
            .filter_entry(|e| is_not_hidden(e) && !is_excluded(e))
            .filter_map(|e| e.ok())
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

fn is_not_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() == 0 || !s.starts_with('.'))
        .unwrap_or(false)
}

fn is_excluded(entry: &DirEntry) -> bool {
    let exclude = vec!["nxcache", "node_modules", "coverage", "dist"];
    entry
        .file_name()
        .to_str()
        .map(|s| exclude.contains(&s))
        .unwrap_or(false)
}
