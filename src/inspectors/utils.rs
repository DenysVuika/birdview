// use serde_json::Value;
// use std::fs::File;
// use std::io::BufReader;
// use std::path::Path;

// pub fn load_json_file(path: &Path) -> Value {
//     let file = File::open(path).unwrap();
//     let reader = BufReader::new(file);
//     let value: Value = serde_json::from_reader(reader).unwrap();
//     value
// }

#[cfg(test)]
pub mod test_utils {
    use crate::inspectors::FileInspectorOptions;
    use assert_fs::NamedTempFile;

    pub fn options_from_file(file: &NamedTempFile) -> FileInspectorOptions {
        let parent = file.parent().unwrap();

        FileInspectorOptions {
            working_dir: parent.to_path_buf(),
            path: file.path().to_path_buf(),
            relative_path: file.path().strip_prefix(parent).unwrap().to_path_buf(),
        }
    }
}
