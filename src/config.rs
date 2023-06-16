use clap::ValueEnum;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Html,
    Json,
}

#[derive(Clone)]
pub struct Config {
    pub working_dir: PathBuf,
    pub output_dir: PathBuf,
    pub verbose: bool,
    pub open: bool,
    pub format: OutputFormat,
}
