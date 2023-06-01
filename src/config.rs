use std::path::PathBuf;

pub struct Config {
    pub working_dir: PathBuf,
    pub inspect_tests: bool,
    pub inspect_deps: bool,
    pub verbose: bool,
    pub output: Option<PathBuf>,
}
