use crate::report::Report;
use std::error::Error;
use std::path::PathBuf;

pub mod fs;
pub mod report;

pub struct Config {
    pub working_dir: PathBuf,
    pub inspect_tests: bool,
    pub inspect_deps: bool,
    pub verbose: bool,
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let report = Report::generate(&config)?;
    report.print(&config.verbose);

    println!("Inspection complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
}
