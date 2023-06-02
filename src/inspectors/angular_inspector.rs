use super::FileInspector;
use crate::workspace::Workspace;
use serde_json::{json, Map, Value};
use std::path::Path;

#[derive(Default)]
pub struct AngularInspector {}

impl AngularInspector {
    pub fn new() -> Self {
        Default::default()
    }
}

impl FileInspector for AngularInspector {
    fn supports_file(&self, path: &Path) -> bool {
        path.is_file() && path.display().to_string().ends_with(".component.ts")
    }

    fn inspect_file(&self, workspace: &Workspace, path: &Path, output: &mut Map<String, Value>) {
        let angular = output
            .entry("angular")
            .or_insert(json!({}))
            .as_object_mut()
            .unwrap();

        let components = angular
            .entry("components")
            .or_insert(json!([]))
            .as_array_mut()
            .unwrap();

        let workspace_path = path
            .strip_prefix(&workspace.working_dir)
            .unwrap()
            .display()
            .to_string();

        // todo: check for the @Component decorator

        let entry = json!({
            "path": workspace_path,
        });

        components.push(entry);
    }

    fn finalize(&self, workspace: &Workspace, output: &mut Map<String, Value>) {
        // println!("Done inspecting angular files");
    }
}
