use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug)]
pub struct Report {
    pub unit_tests: Option<Vec<TestFile>>,
    pub e2e_tests: Option<Vec<TestFile>>,
    pub package_files: Option<Vec<PackageFile>>,
}

impl Report {
    pub fn print(&self, verbose: &bool) {
        if let Some(files) = &self.unit_tests {
            let total_files: usize = files.len();
            let total_tests: usize = files.iter().map(|f| f.test_names.len()).sum();

            if total_files > 0 {
                println!(
                    "unit test files (.spec.ts): {} ({} cases)",
                    total_files, total_tests
                );

                if *verbose {
                    for test_file in files {
                        println!("{}", test_file.file_path);

                        for test_name in &test_file.test_names {
                            println!("  ├── {test_name}");
                        }
                    }
                }
            }
        }

        if let Some(files) = &self.e2e_tests {
            let total_files: usize = files.len();
            let total_tests: usize = files.iter().map(|f| f.test_names.len()).sum();

            if total_files > 0 {
                println!(
                    "e2e test files (.test.ts, .e2e.ts): {} ({} cases)",
                    total_files, total_tests
                );

                if *verbose {
                    for test_file in files {
                        println!("{}", test_file.file_path);

                        for test_name in &test_file.test_names {
                            println!("  ├── {test_name}");
                        }
                    }
                }
            }
        }

        if let Some(files) = &self.package_files {
            let total_files: usize = files.len();

            if total_files > 0 {
                println!("Found package.json files: {}", total_files);

                if *verbose {
                    for package_file in files {
                        println!("{}", package_file.file_path);

                        if package_file.dependencies.len() > 0 {
                            println!("  ├── dependencies");
                            for dependency in &package_file.dependencies {
                                println!("\t├── {}", dependency);
                            }
                        }

                        if package_file.dev_dependencies.len() > 0 {
                            println!("  ├── devDependencies");
                            for dependency in &package_file.dev_dependencies {
                                println!("\t├── {}", dependency);
                            }
                        }
                    }
                }
            }
        }
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

#[derive(Debug)]
pub struct PackageFile {
    pub file_path: String,
    pub dependencies: Vec<String>,
    pub dev_dependencies: Vec<String>,
}

impl PackageFile {
    pub fn from_path(working_dir: &Path, path: &Path) -> Result<PackageFile, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let value: Value = serde_json::from_reader(reader)?;
        let mut dependencies: Vec<String> = Vec::new();
        let mut dev_dependencies: Vec<String> = Vec::new();

        if let Some(data) = value["dependencies"].as_object() {
            for (key, _value) in data {
                // println!("{} {}: {}", path.display(), key, value);
                dependencies.push(key.to_string());
            }
        }

        if let Some(data) = value["devDependencies"].as_object() {
            for (key, _value) in data {
                // println!("{} {}: {}", path.display(), key, value);
                dev_dependencies.push(key.to_string());
            }
        }

        Ok(PackageFile {
            file_path: path.strip_prefix(working_dir)?.display().to_string(),
            dependencies,
            dev_dependencies,
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
