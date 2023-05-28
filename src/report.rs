use std::path::Path;
use std::error::Error;
use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug)]
pub struct Report {
    pub package_files_count: usize,

    pub spec_files: Vec<TestFile>,
    pub test_files: Vec<TestFile>,
}

impl Report {
    pub fn print(&self) {
        let total_spec_cases: usize = self.spec_files.iter().map(|f| f.test_names.len()).sum();
        let total_test_cases: usize = self.test_files.iter().map(|f| f.test_names.len()).sum();

        for test_file in &self.spec_files {
            println!("{}", test_file.file_path);

            for test_name in &test_file.test_names {
                println!("  ├── {test_name}");
            }
        }

        for test_file in &self.test_files {
            println!("{}", test_file.file_path);

            for test_name in &test_file.test_names {
                println!("  ├── {test_name}");
            }
        }

        println!(
            "Found .spec.ts files: {} ({} cases)",
            self.spec_files.len(),
            total_spec_cases
        );
        println!(
            "Found .test.ts files: {} ({} cases)",
            self.test_files.len(),
            total_test_cases
        );
        println!("Found package.json files: {}", self.package_files_count);
    }
}

#[derive(Debug)]
pub struct TestFile {
    pub file_path: String,
    pub test_names: Vec<String>,
}

impl TestFile {
    pub fn from_path(working_dir: &Path, path: &Path) -> Result<TestFile, Box<dyn Error>> {
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

fn extract_test_names(contents: &str) -> Vec<&str> {
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
}
