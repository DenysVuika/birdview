use super::FileInspector;
use crate::workspace::Workspace;
use serde_json::{json, Map, Value};
use std::path::Path;

pub struct AngularInspector {
    components: Vec<String>,
    directives: Vec<String>,
    services: Vec<String>,
    pipes: Vec<String>,
    dialogs: Vec<String>,
}

impl AngularInspector {
    pub fn new() -> Self {
        AngularInspector {
            components: vec![],
            directives: vec![],
            services: vec![],
            pipes: vec![],
            dialogs: vec![],
        }
    }
}

impl Default for AngularInspector {
    fn default() -> Self {
        AngularInspector::new()
    }
}

impl FileInspector for AngularInspector {
    fn supports_file(&self, path: &Path) -> bool {
        let display_path = path.display().to_string();
        path.is_file()
            && (display_path.ends_with(".component.ts")
                || display_path.ends_with(".directive.ts")
                || display_path.ends_with(".service.ts")
                || display_path.ends_with(".pipe.ts")
                || display_path.ends_with(".dialog.ts"))
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

        if workspace_path.ends_with(".component.ts") {
            self.components.push(workspace_path);
        } else if workspace_path.ends_with(".directive.ts") {
            self.directives.push(workspace_path);
        } else if workspace_path.ends_with(".service.ts") {
            self.services.push(workspace_path);
        } else if workspace_path.ends_with(".pipe.ts") {
            self.pipes.push(workspace_path);
        } else if workspace_path.ends_with(".dialog.ts") {
            self.dialogs.push(workspace_path);
        }
    }

    fn finalize(&self, _workspace: &Workspace, output: &mut Map<String, Value>) {
        let angular = output
            .entry("angular")
            .or_insert(json!({
                "components": [],
                "directives": [],
                "services": [],
                "pipes": [],
                "dialogs": [],
            }))
            .as_object_mut()
            .unwrap();

        angular["components"] = json!(self.components);
        angular["directives"] = json!(self.directives);
        angular["services"] = json!(self.services);
        angular["pipes"] = json!(self.pipes);
        angular["dialogs"] = json!(self.dialogs);

        println!("Components: {}", self.components.len());
        println!("Directives: {}", self.directives.len());
        println!("Services: {}", self.services.len());
        println!("Pipes: {}", self.pipes.len());
        println!("Dialogs: {}", self.dialogs.len());
    }
}
