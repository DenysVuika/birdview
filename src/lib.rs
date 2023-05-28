// use std::env;
use lazy_static::lazy_static;
use regex::Regex;
use std::error::Error;
use std::path::Path;
use walkdir::WalkDir;

pub mod fs;

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

#[derive(Debug)]
struct Report {
    package_files_count: usize,

    spec_files: Vec<TestFile>,
    test_files: Vec<TestFile>,
}

#[derive(Debug)]
struct TestFile {
    file_path: String,
    test_names: Vec<String>,
}

impl TestFile {
    fn from_path(working_dir: &Path, path: &Path) -> Result<TestFile, Box<dyn Error>> {
        let contents = std::fs::read_to_string(path)?;
        let test_names: Vec<String> = extract_test_names(&contents)
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        Ok(TestFile {
            file_path: path.strip_prefix(working_dir)?.display().to_string(),
            test_names,
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let working_dir = Path::new(&config.working_dir);
    let report = inspect_dir(working_dir)?;
    print_report(&report);

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

fn print_report(report: &Report) {
    // println!("{:?}", report);

    let total_spec_files: usize = report.spec_files.iter().map(|f| f.test_names.len()).sum();
    let total_test_files: usize = report.test_files.iter().map(|f| f.test_names.len()).sum();

    for test_file in &report.spec_files {
        println!("{}", test_file.file_path);

        for test_name in &test_file.test_names {
            println!("  ├── {test_name}");
        }
    }

    for test_file in &report.test_files {
        println!("{}", test_file.file_path);

        for test_name in &test_file.test_names {
            println!("  ├── {test_name}");
        }
    }

    println!(
        "Found .spec.ts files: {} ({} cases)",
        report.spec_files.len(),
        total_spec_files
    );
    println!(
        "Found .test.ts files: {} ({} cases)",
        report.test_files.len(),
        total_test_files
    );
    println!("Found package.json files: {}", report.package_files_count);
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

pub fn extract_test_names(contents: &str) -> Vec<&str> {
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
