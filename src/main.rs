use std::error::Error;
use std::path::Path;
use std::{env, process};
use walkdir::WalkDir;

use birdview::Config;
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

    // println!("Searching for {}", config.query);
    // println!("In file {}", config.file_path);

    // if let Err(e) = birdview::run(config) {
    //     eprintln!("Application error {e}");
    //     process::exit(1);
    // }
}

fn inspect_dir(working_dir: &Path) -> Result<(), Box<dyn Error>> {
    let walker = WalkDir::new(working_dir).follow_links(true).into_iter();
    let mut spec_files_count = 0;
    let mut test_files_count = 0;
    let mut package_files_count = 0;

    for entry in walker
        .filter_entry(|e| fs::is_not_hidden(e) && !fs::is_excluded(e))
        .filter_map(|e| e.ok())
    {
        let f_name = entry.file_name().to_string_lossy();

        if f_name.ends_with(".spec.ts") {
            spec_files_count += 1;
            // println!("{}", entry.path().display());
        }
        if f_name.ends_with(".test.ts") {
            test_files_count += 1;
        }
        if f_name == "package.json" {
            package_files_count += 1;
        }
    }

    println!("Found .spec.ts files: {}", spec_files_count);
    println!("Found .test.ts files: {}", test_files_count);
    println!("Found package.json files: {}", package_files_count);

    Ok(())
}
