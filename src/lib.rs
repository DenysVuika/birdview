// use std::env;
// use std::error::Error;
use lazy_static::lazy_static;
use regex::Regex;

pub mod fs;

pub struct Config {
    pub working_dir: String,
    // pub query: String,
    // pub file_path: String,
    // pub ignore_case: bool,
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

        Ok(Config {
            working_dir,
            // query,
            // file_path,
            // ignore_case
        })
    }
}

// pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
//     let contents = fs::read_to_string(config.file_path)?;
//
//     let results = if config.ignore_case {
//         search_case_insensitive(&config.query, &contents)
//     } else {
//         search(&config.query, &contents)
//     };
//
//     for line in results {
//         println!("{line}");
//     }
//
//     Ok(())
// }

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
    // (it\(['"])(?P<name>.*?)(['"])
    // https://rustexp.lpil.uk/
    lazy_static! {
        static ref NAME_REGEX: Regex = Regex::new(r#"(it\(['"])(?P<name>.*?)(['"])"#).unwrap();
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
