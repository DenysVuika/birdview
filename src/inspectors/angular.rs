use super::FileInspector;
use crate::db;
use crate::db::NgKind;
use crate::inspectors::FileInspectorOptions;
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use rusqlite::Connection;
use std::path::Path;

pub struct AngularInspector {}

impl AngularInspector {
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

    fn inspect_file(&self, conn: &Connection, opts: &FileInspectorOptions) -> Result<()> {
        let path = &opts.relative_path;
        let sid = opts.sid;
        let url = &opts.url;
        let content = opts.read_content();

        if path.ends_with(".module.ts") {
            db::create_ng_entity(conn, sid, NgKind::Module, path, url, false)?;
        } else if path.ends_with(".component.ts") {
            if content.contains("@Component(") {
                let standalone = AngularInspector::is_standalone(&content);
                db::create_ng_entity(conn, sid, NgKind::Component, path, url, standalone)?;
            }
        } else if path.ends_with(".directive.ts") {
            if content.contains("@Directive(") {
                let standalone = AngularInspector::is_standalone(&content);
                db::create_ng_entity(conn, sid, NgKind::Directive, path, url, standalone)?;
            }
        } else if path.ends_with(".service.ts") {
            db::create_ng_entity(conn, sid, NgKind::Service, path, url, false)?;
        } else if path.ends_with(".pipe.ts") {
            if content.contains("@Pipe(") {
                let standalone = AngularInspector::is_standalone(&content);
                db::create_ng_entity(conn, sid, NgKind::Pipe, path, url, standalone)?;
            }
        } else if path.ends_with(".dialog.ts") && content.contains("@Component(") {
            let standalone = AngularInspector::is_standalone(&content);
            db::create_ng_entity(conn, sid, NgKind::Dialog, path, url, standalone)?;
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
