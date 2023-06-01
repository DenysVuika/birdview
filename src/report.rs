use crate::workspace::{EndToEndTestInspector, PackageJsonInspector, UnitTestInspector, Workspace};
use crate::Config;
use serde::Serialize;
use serde_json::Value;
use std::error::Error;
use std::path::PathBuf;

#[derive(Serialize, Debug)]
pub struct JsonReport {}

#[derive(Debug)]
pub struct Report {}

impl Report {
    pub fn generate(config: &Config) -> Result<Report, Box<dyn Error>> {
        let working_dir = &config.working_dir;

        let workspace = Workspace::setup(
            PathBuf::from(working_dir),
            vec![
                Box::new(PackageJsonInspector {}),
                Box::new(UnitTestInspector {}),
                Box::new(EndToEndTestInspector {}),
            ],
        );

        let output = workspace.inspect();
        let obj = Value::Object(output);
        println!("{}", serde_json::to_string_pretty(&obj).unwrap());

        Ok(Report {})
    }

    pub fn print(&self, verbose: &bool) {}
}

// #[derive(Serialize, Debug)]
// pub struct PackageFile {
//     pub path: String,
//     pub dependencies: Vec<String>,
//     pub dev_dependencies: Vec<String>,
// }
//
// impl PackageFile {
//     pub fn from_path(working_dir: &Path, path: &Path) -> Result<PackageFile, Box<dyn Error>> {
//         let value = Workspace::load_json_file(path);
//         let mut dependencies: Vec<String> = Vec::new();
//         let mut dev_dependencies: Vec<String> = Vec::new();
//
//         if let Some(data) = value["dependencies"].as_object() {
//             for (key, _value) in data {
//                 // println!("{} {}: {}", path.display(), key, value);
//                 dependencies.push(key.to_string());
//             }
//         }
//
//         if let Some(data) = value["devDependencies"].as_object() {
//             for (key, _value) in data {
//                 // println!("{} {}: {}", path.display(), key, value);
//                 dev_dependencies.push(key.to_string());
//             }
//         }
//
//         Ok(PackageFile {
//             path: path.strip_prefix(working_dir)?.display().to_string(),
//             dependencies,
//             dev_dependencies,
//         })
//     }
// }
//
