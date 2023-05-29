use crate::report::{PackageFile, Report, TestFile};
use std::error::Error;
use std::path::PathBuf;
use walkdir::WalkDir;

pub mod fs;
pub mod report;

pub struct Config {
    pub working_dir: PathBuf,
    pub inspect_tests: bool,
    pub inspect_deps: bool,
    pub verbose: bool,
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let report = inspect_dir(&config)?;
    report.print(&config.verbose);

    println!("Inspection complete");
    Ok(())
}

fn inspect_dir(config: &Config) -> Result<Report, Box<dyn Error>> {
    let working_dir = &config.working_dir;
    let walker = WalkDir::new(working_dir).follow_links(true).into_iter();
    let mut spec_files = Vec::new();
    let mut test_files = Vec::new();
    let mut package_files = Vec::new();

    for entry in walker
        .filter_entry(|e| fs::is_not_hidden(e) && !fs::is_excluded(e))
        .filter_map(|e| e.ok())
    {
        let f_name = entry.file_name().to_string_lossy();
        let entry_path = entry.path();

        if config.inspect_tests {
            if f_name.ends_with(".spec.ts") {
                spec_files.push(TestFile::from_path(working_dir, entry_path)?);
            }
            if f_name.ends_with(".test.ts") {
                test_files.push(TestFile::from_path(working_dir, entry_path)?);
            }
        }

        if config.inspect_deps {
            if f_name == "package.json" {
                package_files.push(PackageFile::from_path(working_dir, entry_path)?)
            }
        }
    }

    Ok(Report {
        spec_files: Some(spec_files),
        test_files: Some(test_files),
        package_files: Some(package_files),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
}
