use std::error::Error;
use std::path::Path;
use std::{env, process};
use walkdir::WalkDir;

use birdview::{Config, extract_test_names};
use birdview::fs;

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    println!("Working dir: {}", config.working_dir);
    let working_dir = Path::new(&config.working_dir);

    if let Err(e) = inspect_dir(working_dir) {
        eprintln!("Application error {e}");
        process::exit(1);
    }
}

fn inspect_dir(working_dir: &Path) -> Result<(), Box<dyn Error>> {
    let walker = WalkDir::new(working_dir).follow_links(true).into_iter();
    let mut spec_files_count = 0;
    let mut test_files_count = 0;
    let mut package_files_count = 0;
    let mut total_unit_tests_count = 0;
    let mut total_e2e_tests_count = 0;

    for entry in walker
        .filter_entry(|e| fs::is_not_hidden(e) && !fs::is_excluded(e))
        .filter_map(|e| e.ok())
    {
        let f_name = entry.file_name().to_string_lossy();

        if f_name.ends_with(".spec.ts") {
            spec_files_count += 1;
            println!("{}", entry.path().strip_prefix(working_dir)?.display());

            let contents = std::fs::read_to_string(entry.path())?;
            for test_name in extract_test_names(&contents) {
                total_unit_tests_count += 1;
                println!("  ├── {test_name}");
            }

        }
        if f_name.ends_with(".test.ts") {
            test_files_count += 1;
            println!("{}", entry.path().strip_prefix(working_dir)?.display());

            let contents = std::fs::read_to_string(entry.path())?;
            for test_name in extract_test_names(&contents) {
                total_e2e_tests_count += 1;
                println!("  ├── {test_name}");
            }
        }

        if f_name == "package.json" {
            package_files_count += 1;
        }
    }

    println!("Found .spec.ts files: {} ({} cases)", spec_files_count, total_unit_tests_count);
    println!("Found .test.ts files: {}, ({} cases)", test_files_count, total_e2e_tests_count);
    println!("Found package.json files: {}", package_files_count);

    Ok(())
}
