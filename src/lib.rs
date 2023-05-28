use std::error::Error;
use std::path::Path;
use walkdir::WalkDir;
use crate::report::{Report, TestFile};

pub mod fs;
pub mod report;

pub struct Config {
    pub working_dir: String,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let working_dir = match args.next() {
            Some(arg) => arg,
            None => return Err("Missing working directory"),
        };

        // let query = match args.next() {
        //     Some(arg) => arg,
        //     None => return Err("Didn't find a query string")
        // };
        //
        // let file_path = match args.next() {
        //     Some(arg) => arg,
        //     None => return Err("Didn't get a file path")
        // };

        // let ignore_case = env::var("IGNORE_CASE").is_ok();

        Ok(Config { working_dir })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let working_dir = Path::new(&config.working_dir);
    let report = inspect_dir(working_dir)?;
    report.print();

    // let contents = fs::read_to_string(config.file_path)?;

    // let results = if config.ignore_case {
    //     search_case_insensitive(&config.query, &contents)
    // } else {
    //     search(&config.query, &contents)
    // };
    //
    // for line in results {
    //     println!("{line}");
    // }

    Ok(())
}

fn inspect_dir(working_dir: &Path) -> Result<Report, Box<dyn Error>> {
    let walker = WalkDir::new(working_dir).follow_links(true).into_iter();
    let mut package_files_count = 0;
    let mut spec_files = Vec::new();
    let mut test_files = Vec::new();

    for entry in walker
        .filter_entry(|e| fs::is_not_hidden(e) && !fs::is_excluded(e))
        .filter_map(|e| e.ok())
    {
        let f_name = entry.file_name().to_string_lossy();

        if f_name.ends_with(".spec.ts") {
            spec_files.push(TestFile::from_path(working_dir, entry.path())?);
        }
        if f_name.ends_with(".test.ts") {
            test_files.push(TestFile::from_path(working_dir, entry.path())?);
        }

        if f_name == "package.json" {
            package_files_count += 1;
        }
    }

    Ok(Report {
        package_files_count,
        spec_files,
        test_files,
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
