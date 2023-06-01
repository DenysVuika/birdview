use crate::report::{JsonReport, Report};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub mod fs;
pub mod report;
pub mod workspace;

pub struct Config {
    pub working_dir: PathBuf,
    pub inspect_tests: bool,
    pub inspect_deps: bool,
    pub verbose: bool,
    pub output: Option<PathBuf>,
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let report = Report::generate(&config)?;
    report.print(&config.verbose);

    if let Some(output) = config.output {
        let json_report = JsonReport {};

        let serialized = serde_json::to_string_pretty(&json_report).unwrap();
        let mut output_file = File::create(&output)?;
        write!(output_file, "{}", serialized)?;

        println!("Saved full report to {}", &output.display());
    }

    println!("Inspection complete");
    Ok(())
}
