use crate::report::{PackageFile, Report, TestFile};
use std::error::Error;
use std::path::Path;
use walkdir::WalkDir;

pub mod fs;
pub mod report;

pub fn run(working_dir: &Path) -> Result<(), Box<dyn Error>> {
    let report = inspect_dir(working_dir)?;
    report.print();

    Ok(())
}

fn inspect_dir(working_dir: &Path) -> Result<Report, Box<dyn Error>> {
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

        if f_name.ends_with(".spec.ts") {
            spec_files.push(TestFile::from_path(working_dir, entry_path)?);
        }
        if f_name.ends_with(".test.ts") {
            test_files.push(TestFile::from_path(working_dir, entry_path)?);
        }

        if f_name == "package.json" {
            package_files.push(PackageFile::from_path(working_dir, entry_path)?)
        }
    }

    Ok(Report {
        spec_files: Some(spec_files),
        test_files: Some(test_files),
        package_files: Some(package_files),
    })
}

pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents
        .lines()
        .filter(|line| line.contains(query))
        .collect()
}

pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase();
    contents
        .lines()
        .filter(|line| line.to_lowercase().contains(&query))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_sensitive() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";
        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }

    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(
            vec!["Rust:", "Trust me."],
            search_case_insensitive(query, contents)
        );
    }
}
