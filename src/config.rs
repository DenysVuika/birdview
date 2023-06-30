use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
    pub project_name: String,
    pub working_dir: PathBuf,
    pub output_dir: PathBuf,
    pub verbose: bool,
    pub open: bool,
    pub tags: bool,
}
