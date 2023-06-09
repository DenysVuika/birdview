use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageJsonFile {
    pub name: Option<String>,
    pub version: Option<String>,

    pub dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Option<HashMap<String, String>>,
}

impl PackageJsonFile {
    /// Read the `package.json` file
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let result = serde_json::from_reader(reader)?;
        Ok(result)
    }
}
