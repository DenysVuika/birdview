use super::FileInspector;
use crate::workspace::Workspace;
use serde_json::{json, Map, Value};
use std::path::Path;

pub struct AngularInspector {
    modules: Vec<String>,
    components: Vec<String>,
    directives: Vec<String>,
    services: Vec<String>,
    pipes: Vec<String>,
    dialogs: Vec<String>,
}

impl AngularInspector {
    pub fn new() -> Self {
        AngularInspector {
            modules: vec![],
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
            && (display_path.ends_with(".module.ts")
                || display_path.ends_with(".component.ts")
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

        if workspace_path.ends_with(".module.ts") {
            self.modules.push(workspace_path);
        } else if workspace_path.ends_with(".component.ts") {
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

    fn finalize(&mut self, _workspace: &Workspace, output: &mut Map<String, Value>) {
        output.entry("angular").or_insert(json!({
            "modules": self.modules,
            "components": self.components,
            "directives": self.directives,
            "services": self.services,
            "pipes": self.pipes,
            "dialogs": self.dialogs
        }));

        let stats = output
            .entry("stats")
            .or_insert(json!({}))
            .as_object_mut()
            .unwrap();

        stats.entry("angular").or_insert(json!({
            "module": self.modules.len(),
            "component": self.components.len(),
            "directive": self.directives.len(),
            "service": self.services.len(),
            "pipe": self.pipes.len(),
            "dialog": self.dialogs.len()
        }));

        println!("Angular");
        println!(" ├── Module: {}", self.modules.len());
        println!(" ├── Component: {}", self.components.len());
        println!(" ├── Directive: {}", self.directives.len());
        println!(" ├── Service: {}", self.services.len());
        println!(" ├── Pipe: {}", self.pipes.len());
        println!(" └── Dialog: {}", self.dialogs.len());

        // cleanup
        self.modules = Vec::new();
        self.components = Vec::new();
        self.directives = Vec::new();
        self.services = Vec::new();
        self.pipes = Vec::new();
        self.dialogs = Vec::new();
    }
}
