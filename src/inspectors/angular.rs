use super::FileInspector;
use crate::inspectors::FileInspectorOptions;
use lazy_static::lazy_static;
use regex::Regex;
use rusqlite::{params, Connection};
use serde_json::{Map, Value};
use std::error::Error;
use std::path::Path;
use uuid::Uuid;

pub struct AngularInspector {}

impl AngularInspector {
    pub fn new() -> Self {
        AngularInspector {}
    }

    fn extract_metadata(contents: &str) -> Vec<&str> {
        // @(?:Component|Directive|Injectable)\((?P<metadata>[^\)]+)\)
        // https://rustexp.lpil.uk/
        lazy_static! {
            static ref NAME_REGEX: Regex =
                Regex::new(r#"@(?:Component|Directive|Pipe|Injectable)\((?P<metadata>[^\)]+)\)"#)
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
        connection: &Connection,
        project_id: &Uuid,
        options: &FileInspectorOptions,
        _output: &mut Map<String, Value>,
    ) -> Result<(), Box<dyn Error>> {
        let workspace_path = options.relative_path.display().to_string();
        let content = options.read_content();

        if workspace_path.ends_with(".module.ts") {
            connection.execute(
                "INSERT INTO ng_modules (id, project_id, path) VALUES (?1, ?2, ?3)",
                params![Uuid::new_v4(), project_id, workspace_path],
            )?;
        } else if workspace_path.ends_with(".component.ts") {
            if content.contains("@Component(") {
                let standalone = AngularInspector::is_standalone(&content);

                connection.execute(
                "INSERT INTO ng_components (id, project_id, path, standalone) VALUES (?1, ?2, ?3, ?4)",
                params![Uuid::new_v4(), project_id, workspace_path, standalone],
                )?;
            }
        } else if workspace_path.ends_with(".directive.ts") {
            if content.contains("@Directive(") {
                let standalone = AngularInspector::is_standalone(&content);

                connection.execute(
                    "INSERT INTO ng_directives (id, project_id, path, standalone) VALUES (?1, ?2, ?3, ?4)",
                    params![Uuid::new_v4(), project_id, workspace_path, standalone],
                )?;
            }
        } else if workspace_path.ends_with(".service.ts") {
            connection.execute(
                "INSERT INTO ng_services (id, project_id, path) VALUES (?1, ?2, ?3)",
                params![Uuid::new_v4(), project_id, workspace_path],
            )?;
        } else if workspace_path.ends_with(".pipe.ts") {
            if content.contains("@Pipe(") {
                let standalone = AngularInspector::is_standalone(&content);

                connection.execute(
                    "INSERT INTO ng_pipes (id, project_id, path, standalone) VALUES (?1, ?2, ?3, ?4)",
                    params![Uuid::new_v4(), project_id, workspace_path, standalone],
                )?;
            }
        } else if workspace_path.ends_with(".dialog.ts") && content.contains("@Component(") {
            let standalone = AngularInspector::is_standalone(&content);

            connection.execute(
                "INSERT INTO ng_dialogs (id, project_id, path, standalone) VALUES (?1, ?2, ?3, ?4)",
                params![Uuid::new_v4(), project_id, workspace_path, standalone],
            )?;
        }

        Ok(())
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
