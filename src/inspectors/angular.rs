use super::FileInspector;
use crate::inspectors::utils::load_json_file;
use crate::inspectors::FileInspectorOptions;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct AngularComponent {
    path: String,
    standalone: bool,
}

#[derive(Serialize, Deserialize)]
pub struct AngularEntity {
    path: String,
}

pub struct AngularInspector {
    framework: Option<String>,
    modules: Vec<AngularEntity>,
    components: Vec<AngularComponent>,
    directives: Vec<AngularEntity>,
    services: Vec<AngularEntity>,
    pipes: Vec<AngularEntity>,
    dialogs: Vec<AngularEntity>,
}

impl AngularInspector {
    pub fn new() -> Self {
        AngularInspector {
            framework: None,
            modules: vec![],
            components: vec![],
            directives: vec![],
            services: vec![],
            pipes: vec![],
            dialogs: vec![],
        }
    }

    fn extract_metadata(contents: &str) -> Vec<&str> {
        // @(?:Component|Directive|Injectable)\((?P<metadata>[^\)]+)\)
        // https://rustexp.lpil.uk/
        lazy_static! {
            static ref NAME_REGEX: Regex =
                Regex::new(r#"@(?:Component|Directive|Injectable)\((?P<metadata>[^\)]+)\)"#)
                    .unwrap();
        }

        NAME_REGEX
            .captures_iter(contents)
            .map(|c| c.name("metadata").unwrap().as_str())
            .collect()
    }

    fn is_standalone(content: &str) -> bool {
        let mut standalone = false;
        let metadata = AngularInspector::extract_metadata(content);

        if !metadata.is_empty() {
            let mut parsed = metadata.first().unwrap().to_string();
            parsed = parsed.replace('\n', "");
            parsed = parsed.replace(' ', "");
            standalone = parsed.contains("standalone:true");
        }

        standalone
    }

    fn get_angular_version(working_dir: &Path) -> Option<String> {
        let package_path = &working_dir.join("package.json");

        if package_path.exists() {
            let content = load_json_file(package_path);
            if let Some(data) = content["dependencies"].as_object() {
                if let Some(version) = data.get("@angular/core") {
                    let result: String = version.as_str().unwrap().to_string();
                    return Some(result);
                }
            }
        }

        None
    }
}

impl Default for AngularInspector {
    fn default() -> Self {
        AngularInspector::new()
    }
}

impl FileInspector for AngularInspector {
    fn get_module_name(&self) -> &str {
        "angular-entities"
    }

    fn init(&mut self, working_dir: &Path, _output: &mut Map<String, Value>) {
        self.framework = AngularInspector::get_angular_version(working_dir);
    }

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

    fn inspect_file(&mut self, options: &FileInspectorOptions, _output: &mut Map<String, Value>) {
        let workspace_path = options.relative_path.display().to_string();

        if workspace_path.ends_with(".module.ts") {
            self.modules.push(AngularEntity {
                path: workspace_path,
            });
        } else if workspace_path.ends_with(".component.ts") {
            let content = options.read_content();
            if content.contains("@Component(") {
                let standalone = AngularInspector::is_standalone(&content);
                self.components.push(AngularComponent {
                    path: workspace_path,
                    standalone,
                });
            }
        } else if workspace_path.ends_with(".directive.ts") {
            self.directives.push(AngularEntity {
                path: workspace_path,
            });
        } else if workspace_path.ends_with(".service.ts") {
            self.services.push(AngularEntity {
                path: workspace_path,
            });
        } else if workspace_path.ends_with(".pipe.ts") {
            self.pipes.push(AngularEntity {
                path: workspace_path,
            });
        } else if workspace_path.ends_with(".dialog.ts") {
            self.dialogs.push(AngularEntity {
                path: workspace_path,
            });
        }
    }

    fn finalize(&mut self, output: &mut Map<String, Value>) {
        let framework = match &self.framework {
            Some(value) => value,
            None => "unknown",
        };

        output.entry("angular").or_insert(json!({
            "framework": framework,
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

        let standalone_components = self
            .components
            .iter()
            .filter(|entry| entry.standalone)
            .count();

        stats.entry("angular").or_insert(json!({
            "module": self.modules.len(),
            "component": self.components.len(),
            "component_standalone": standalone_components,
            "directive": self.directives.len(),
            "service": self.services.len(),
            "pipe": self.pipes.len(),
            "dialog": self.dialogs.len()
        }));

        println!("Angular");
        println!(" ├── Framework: {}", framework);
        println!(" ├── Module: {}", self.modules.len());
        println!(
            " ├── Component: {} (standalone: {})",
            self.components.len(),
            standalone_components
        );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_single_metadata() {
        let content = r#"@Component({ selector: 'my-component' }) export class MyComponent {}"#;
        let metadata = AngularInspector::extract_metadata(content);
        assert_eq!(metadata.len(), 1);
        assert_eq!(
            metadata.first().unwrap().to_string(),
            "{ selector: 'my-component' }"
        );
    }

    #[test]
    fn should_parse_single_metadata_multiline() {
        let content = r#"
            @Component({ 
                selector: 'my-component',
                standalone: true
            }) 
            export class MyComponent {}
        "#;
        let metadata = AngularInspector::extract_metadata(content);
        assert_eq!(metadata.len(), 1);

        let mut parsed = metadata.first().unwrap().to_string();
        parsed = parsed.replace('\n', "");
        parsed = parsed.replace(' ', "");

        assert_eq!(parsed, "{selector:'my-component',standalone:true}");
    }

    #[test]
    fn should_parse_multiple_metadata_entries() {
        let content = r#"
            // component
            @Component({ selector: 'my-component' }) export class MyComponent {}
            
            // directive
            @Directive({ selector: 'my-directive' })
            export class MyDirective {}
            
            // service
            @Injectable({ provideIn: 'root' })
            export class MyService {}
        "#;
        let metadata = AngularInspector::extract_metadata(content);
        assert_eq!(metadata.len(), 3);
        assert_eq!(
            metadata.first().unwrap().to_string(),
            "{ selector: 'my-component' }"
        );
        assert_eq!(
            metadata.get(1).unwrap().to_string(),
            "{ selector: 'my-directive' }"
        );
        assert_eq!(
            metadata.get(2).unwrap().to_string(),
            "{ provideIn: 'root' }"
        )
    }
}
