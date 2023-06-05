use crate::inspectors::{FileInspector, FileInspectorOptions};
use chrono::Utc;
use serde_json::{Map, Value};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

pub struct Workspace {
    pub working_dir: PathBuf,
    verbose: bool,
}

impl Workspace {
    pub fn setup(working_dir: PathBuf, verbose: bool) -> Workspace {
        Workspace {
            working_dir,
            verbose,
        }
    }

    pub fn inspect(
        &self,
        mut inspectors: Vec<Box<dyn FileInspector>>,
    ) -> Result<Value, Box<dyn Error>> {
        let mut map = Map::new();
        if self.verbose {
            println!("{}", self.working_dir.display());
        }

        let package_json_path = &self.working_dir.join("package.json");
        if package_json_path.exists() {
            if self.verbose {
                println!(
                    "├── {}",
                    package_json_path.strip_prefix(&self.working_dir)?.display()
                );
            }
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

        self.run_inspectors(&mut inspectors, &mut map);
        Ok(Value::Object(map))
    }

    fn run_inspectors(
        &self,
        inspectors: &mut Vec<Box<dyn FileInspector>>,
        map: &mut Map<String, Value>,
    ) {
        let walker = WalkDir::new(&self.working_dir)
            .follow_links(true)
            .into_iter();

        for entry in walker
            .filter_entry(|e| is_not_hidden(e) && !is_excluded(e))
            .filter_map(|e| e.ok())
        {
            // let f_name = entry.file_name().to_string_lossy();
            let entry_path = entry.path();

            if self.verbose {
                println!(
                    "├── 🔎 {}",
                    entry_path
                        .strip_prefix(&self.working_dir)
                        .unwrap()
                        .display()
                );
            }

            let options = FileInspectorOptions {
                working_dir: self.working_dir.to_path_buf(),
                path: entry_path.to_path_buf(),
                relative_path: entry_path
                    .strip_prefix(&self.working_dir)
                    .unwrap()
                    .to_path_buf(),
            };

            for inspector in inspectors.iter_mut() {
                if entry_path.is_file() && inspector.supports_file(entry_path) {
                    if self.verbose {
                        println!(
                            "├── ✅  {}",
                            entry_path
                                .strip_prefix(&self.working_dir)
                                .unwrap()
                                .display()
                        );
                    }

                    inspector.inspect_file(&options, map);
                }
            }
        }

        for inspector in inspectors {
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
