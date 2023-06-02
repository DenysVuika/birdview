use super::FileInspector;
use crate::workspace::Workspace;
use serde_json::{json, Map, Value};
use std::path::Path;

pub struct AngularInspector {
    components: Vec<String>,
}

impl AngularInspector {
    pub fn new() -> Self {
        AngularInspector { components: vec![] }
    }
}

impl Default for AngularInspector {
    fn default() -> Self {
        AngularInspector::new()
    }
}

impl FileInspector for AngularInspector {
    fn supports_file(&self, path: &Path) -> bool {
        path.is_file() && path.display().to_string().ends_with(".component.ts")
    }

    fn inspect_file(
        &mut self,
        workspace: &Workspace,
        path: &Path,
        _output: &mut Map<String, Value>,
    ) {
        let workspace_path = path
            .strip_prefix(&workspace.working_dir)
            .unwrap()
            .display()
            .to_string();

        // todo: check for the @Component decorator
        self.components.push(workspace_path.to_string());
    }

    fn finalize(&self, _workspace: &Workspace, output: &mut Map<String, Value>) {
        let angular = output
            .entry("angular")
            .or_insert(json!({}))
            .as_object_mut()
            .unwrap();

        angular["components"] = json!(self.components);
    }
}
