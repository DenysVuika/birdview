use super::FileInspector;
use crate::inspectors::FileInspectorOptions;
use lazy_static::lazy_static;
use regex::Regex;
use rusqlite::{named_params, params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::error::Error;
use std::path::Path;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct AngularDirective {
    path: String,
    standalone: bool,
}

#[derive(Serialize, Deserialize)]
pub struct AngularFile {
    path: String,
}

pub struct AngularInspector {
    // framework: Option<String>,
}

impl AngularInspector {
    pub fn new() -> Self {
        AngularInspector {
            // framework: None,
            // modules: vec![],
            // components: vec![],
            // directives: vec![],
            // services: vec![],
            // pipes: vec![],
            // dialogs: vec![],
        }
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

    fn get_angular_report(
        connection: &Connection,
        project_id: &Uuid,
    ) -> Result<Value, Box<dyn Error>> {
        let mut stmt =
            connection.prepare("SELECT path FROM ng_modules WHERE project_id=:project_id;")?;
        let rows = stmt.query_map(named_params! { ":project_id": project_id }, |row| {
            Ok(AngularFile {
                path: row.get(0).unwrap(),
            })
        })?;
        let modules: Vec<AngularFile> = rows.filter_map(|entry| entry.ok()).collect();

        let mut stmt = connection
            .prepare("SELECT path, standalone FROM ng_components WHERE project_id=:project_id;")?;
        let rows = stmt.query_map(named_params! { ":project_id": project_id }, |row| {
            Ok(AngularDirective {
                path: row.get(0)?,
                standalone: row.get(1)?,
            })
        })?;
        let components: Vec<AngularDirective> = rows.filter_map(|entry| entry.ok()).collect();

        let mut stmt = connection
            .prepare("SELECT path, standalone FROM ng_directives WHERE project_id=:project_id;")?;
        let rows = stmt.query_map(named_params! { ":project_id": project_id }, |row| {
            Ok(AngularDirective {
                path: row.get(0)?,
                standalone: row.get(1)?,
            })
        })?;
        let directives: Vec<AngularDirective> = rows.filter_map(|entry| entry.ok()).collect();

        let mut stmt =
            connection.prepare("SELECT path FROM ng_services WHERE project_id=:project_id;")?;
        let rows = stmt.query_map(named_params! { ":project_id": project_id }, |row| {
            Ok(AngularFile {
                path: row.get(0).unwrap(),
            })
        })?;
        let services: Vec<AngularFile> = rows.filter_map(|entry| entry.ok()).collect();

        let mut stmt = connection
            .prepare("SELECT path, standalone FROM ng_pipes WHERE project_id=:project_id;")?;
        let rows = stmt.query_map(named_params! { ":project_id": project_id }, |row| {
            Ok(AngularDirective {
                path: row.get(0)?,
                standalone: row.get(1)?,
            })
        })?;
        let pipes: Vec<AngularDirective> = rows.filter_map(|entry| entry.ok()).collect();

        let mut stmt = connection
            .prepare("SELECT path, standalone FROM ng_dialogs WHERE project_id=:project_id;")?;
        let rows = stmt.query_map(named_params! { ":project_id": project_id }, |row| {
            Ok(AngularDirective {
                path: row.get(0)?,
                standalone: row.get(1)?,
            })
        })?;
        let dialogs: Vec<AngularDirective> = rows.filter_map(|entry| entry.ok()).collect();

        Ok(json!({
            "modules": modules,
            "components": components,
            "directives": directives,
            "services": services,
            "pipes": pipes,
            "dialogs": dialogs
        }))
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

    fn finalize(
        &mut self,
        connection: &Connection,
        project_id: &Uuid,
        output: &mut Map<String, Value>,
    ) -> Result<(), Box<dyn Error>> {
        // let framework = match &self.framework {
        //     Some(value) => value,
        //     None => "unknown",
        // };

        let angular = AngularInspector::get_angular_report(connection, project_id)?;
        output.entry("angular").or_insert(angular);

        // output.entry("angular").or_insert(json!({
        //     "framework": framework,
        //     "modules": self.modules,
        //     "components": self.components,
        //     "directives": self.directives,
        //     "services": self.services,
        //     "pipes": self.pipes,
        //     "dialogs": self.dialogs
        // }));

        let stats = output
            .entry("stats")
            .or_insert(json!({}))
            .as_object_mut()
            .unwrap();

        // stats.entry("angular").or_insert(json!({
        //     "module": self.modules.len(),
        //     "component": self.components.len(),
        //     "directive": self.directives.len(),
        //     "service": self.services.len(),
        //     "pipe": self.pipes.len(),
        //     "dialog": self.dialogs.len()
        // }));

        // println!("Angular");
        // println!(" ├── Framework: {}", framework);
        // println!(" ├── Module: {}", self.modules.len());
        // println!(" ├── Component: {}", self.components.len());
        // println!(" ├── Directive: {}", self.directives.len());
        // println!(" ├── Service: {}", self.services.len());
        // println!(" ├── Pipe: {}", self.pipes.len());
        // println!(" └── Dialog: {}", self.dialogs.len());

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
