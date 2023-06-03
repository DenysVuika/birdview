pub mod config;
pub mod inspectors;
pub mod workspace;

use crate::config::Config;
use crate::inspectors::{
    AngularInspector, EndToEndTestInspector, FileInspector, MarkdownInspector,
    PackageJsonInspector, UnitTestInspector,
};
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
        inspectors.push(Box::new(UnitTestInspector::new()));
        inspectors.push(Box::new(EndToEndTestInspector::new()));
    }
    if config.inspect_angular {
        inspectors.push(Box::new(AngularInspector::new()));
    }
    if config.inspect_markdown {
        inspectors.push(Box::new(MarkdownInspector::new()));
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
        write!(output_file, "{}", json_string)?;
        println!("Saved report to: {}", &output_path.display());
    }

    println!("Inspection complete");
    Ok(())
}
