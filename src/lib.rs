use crate::workspace::{EndToEndTestInspector, PackageJsonInspector, UnitTestInspector, Workspace};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub mod fs;
pub mod workspace;

pub struct Config {
    pub working_dir: PathBuf,
    pub inspect_tests: bool,
    pub inspect_deps: bool,
    pub verbose: bool,
    pub output: Option<PathBuf>,
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let working_dir = &config.working_dir;

    let workspace = Workspace::setup(
        PathBuf::from(working_dir),
        vec![
            Box::new(PackageJsonInspector {}),
            Box::new(UnitTestInspector {}),
            Box::new(EndToEndTestInspector {}),
        ],
    );

    let output = workspace.inspect();
    let json_string = serde_json::to_string_pretty(&output).unwrap();
    println!("{}", json_string);

    if let Some(output) = config.output {
        let mut output_file = File::create(&output)?;
        write!(output_file, "{}", json_string)?;
        println!("Saved full report to {}", &output.display());
    }

    println!("Inspection complete");
    Ok(())
}
