use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
    pub inspect_tests: bool,
    pub inspect_packages: bool,
    pub verbose: bool,
    pub output: Option<PathBuf>,
}
