use crate::report::{JsonReport, Report};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::{Path, PathBuf};

pub mod fs;
pub mod report;

pub struct Config {
    pub working_dir: PathBuf,
    pub inspect_tests: bool,
    pub inspect_deps: bool,
    pub verbose: bool,
    pub output: Option<PathBuf>,
}

#[derive(Deserialize, Debug)]
struct PackageJson {
    name: String,
    version: String,

    scripts: Option<HashMap<String, String>>,
    dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    dev_dependencies: Option<HashMap<String, String>>,
}

impl PackageJson {
    fn from_path(path: PathBuf) -> Result<PackageJson, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let report = Report::generate(&config)?;
    report.print(&config.verbose);

    let mut project_name = "".to_owned();
    let mut project_version = "".to_owned();

    let package_json_path = Path::new(&config.working_dir).join("package.json");
    if package_json_path.exists() {
        println!("Found root package.json file");
        let package_json = PackageJson::from_path(package_json_path)?;

        project_name = package_json.name;
        project_version = package_json.version;

        if let Some(scripts) = package_json.scripts {
            println!("  ├── scripts: {}", scripts.len());
        }
        if let Some(dependencies) = package_json.dependencies {
            println!("  ├── dependencies: {}", dependencies.len());
        }
        if let Some(dependencies) = package_json.dev_dependencies {
            println!("  ├── devDependencies: {}", dependencies.len());
        }
    }

    if let Some(output) = config.output {
        let json_report = JsonReport {
            project_name,
            project_version,
            unit_tests: report.unit_tests,
            e2e_tests: report.e2e_tests,
            packages: report.package_files,
        };

        let serialized = serde_json::to_string_pretty(&json_report).unwrap();
        let mut output_file = File::create(&output)?;
        write!(output_file, "{}", serialized)?;

        println!("Saved full report to {}", &output.display());
    }

    println!("Inspection complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
}
