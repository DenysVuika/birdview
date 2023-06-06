pub mod config;
pub mod inspectors;
pub mod workspace;

use crate::config::Config;
use crate::inspectors::*;
use crate::workspace::Workspace;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn run(config: &Config, working_dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    let mut inspectors: Vec<Box<dyn FileInspector>> = Vec::new();

    if config.inspect_packages {
        inspectors.push(Box::new(PackageJsonInspector::new()));
    }
    if config.inspect_tests {
        inspectors.push(Box::new(TestInspector::new()));
    }
    if config.inspect_angular {
        inspectors.push(Box::new(AngularInspector::new()));
    }
    if config.inspect_types {
        inspectors.push(Box::new(FileTypeInspector::new()));
    }

    if inspectors.is_empty() {
        println!("No inspectors defined.\nRun 'birdview inspect --help' for available options.");
        return Ok(());
    }

    let workspace = Workspace::setup(working_dir.to_owned(), config.verbose);

    let output = workspace.inspect(inspectors)?;

    if let Some(output_path) = &config.output {
        let mut output_file = File::create(output_path)?;
        let json_string = serde_json::to_string_pretty(&output)?;
        let extension = output_path.extension().unwrap();

        if extension == "json" {
            write!(output_file, "{}", json_string)?;
            println!("Saved report to: {}", &output_path.display());
        } else if extension == "html" {
            let template = include_str!("assets/html/index.html");
            let data = format!("window.data = {};", json_string);
            let template = template.replace("// <birdview:DATA>", &data);

            write!(output_file, "{}", template)?;
            println!("Saved report to: {}", &output_path.display());

            if config.open {
                webbrowser::open(&output_path.display().to_string())?
            }
        }
    }

    println!("Inspection complete");
    Ok(())
}
