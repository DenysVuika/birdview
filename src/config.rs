use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
    pub working_dir: PathBuf,
    pub output_dir: PathBuf,
    pub verbose: bool,
    pub open: bool,
}
