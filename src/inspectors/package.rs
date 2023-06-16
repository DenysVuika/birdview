use super::FileInspector;
use crate::db;
use crate::inspectors::FileInspectorOptions;
use crate::models::PackageJsonFile;
use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

pub struct PackageJsonInspector {}

impl FileInspector for PackageJsonInspector {
    fn supports_file(&self, path: &Path) -> bool {
        path.is_file() && path.ends_with("package.json")
    }

    fn inspect_file(&self, conn: &Connection, opts: &FileInspectorOptions) -> Result<()> {
        let package = PackageJsonFile::from_file(&opts.path)
            // todo: convert to db warning instead
            .unwrap_or_else(|_| panic!("Error reading {}", &opts.path.display()));

        let path = &opts.relative_path;
        let project_id = opts.project_id;
        let url = &opts.url;

        if package.name.is_none() {
            db::create_warning(conn, project_id, path, "Missing [name] attribute", url)?;
        }

        if package.version.is_none() {
            db::create_warning(conn, project_id, path, "Missing [version] attribute", url)?;
        }

        db::create_package(conn, project_id, path, url, &package)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use assert_fs::NamedTempFile;
    use std::error::Error;

    #[test]
    fn supports_json_file() -> Result<(), Box<dyn Error>> {
        let file = NamedTempFile::new("package.json")?;
        file.touch()?;
        let inspector = PackageJsonInspector {};
        assert!(inspector.supports_file(file.path()));

        file.close()?;
        Ok(())
    }

    #[test]
    fn requires_json_file_to_exist() {
        let path = Path::new("missing/package.json");
        let inspector = PackageJsonInspector {};
        assert!(!inspector.supports_file(path));
    }

    /*
    fn parses_package_dependencies() -> Result<(), Box<dyn Error>> {
        let conn = Connection::open_in_memory()?;
        let project_id = Uuid::new_v4();
        let file = NamedTempFile::new("package.json")?;
        file.write_str(
            r#"
            {
                "name": "test",
                "version": "1.0.0",
                "dependencies": {
                    "tslib": "^2.5.0"
                },
                "devDependencies": {
                    "@angular/cli": "14.1.3"
                }
            }
        "#,
        )?;

        let mut inspector = PackageJsonInspector::new();

        let options = options_from_file(&file);

        inspector.inspect_file(&conn, &project_id, &options)?;
        // inspector.finalize(&conn, &project_id, &mut map)?;

        assert_eq!(
            Value::Object(map),
            json!({
                "warnings": [],
                "packages": [
                    {
                        "path": "package.json",
                        "dependencies": [
                            {
                              "name": "tslib",
                              "version": "^2.5.0",
                              "dev": false
                            },
                            {
                              "name": "@angular/cli",
                              "version": "14.1.3",
                              "dev": true
                            }
                        ]
                    }
                ],
                "stats": {
                    "package": {
                        "files": 1,
                        "prod_deps": 1,
                        "dev_deps": 1
                    }
                }
            })
        );

        file.close()?;
        Ok(())
    }
    */
}
