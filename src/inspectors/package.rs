use super::FileInspector;
use crate::inspectors::FileInspectorOptions;
use crate::models::PackageJsonFile;
use rusqlite::{params, Connection};
use std::error::Error;
use std::path::Path;
use uuid::Uuid;

pub struct PackageJsonInspector {}

impl PackageJsonInspector {
    /// Creates a new instance of the inspector
    pub fn new() -> Self {
        PackageJsonInspector {}
    }
}

impl Default for PackageJsonInspector {
    fn default() -> Self {
        PackageJsonInspector::new()
    }
}

impl FileInspector for PackageJsonInspector {
    fn supports_file(&self, path: &Path) -> bool {
        path.is_file() && path.ends_with("package.json")
    }

    fn inspect_file(
        &mut self,
        connection: &Connection,
        project_id: &Uuid,
        options: &FileInspectorOptions,
    ) -> Result<(), Box<dyn Error>> {
        let package = PackageJsonFile::from_file(&options.path)
            // todo: convert to db warning instead
            .unwrap_or_else(|_| panic!("Error reading {}", &options.path.display()));

        let workspace_path = options.relative_path.display().to_string();

        if package.name.is_none() {
            connection.execute(
                "INSERT INTO warnings (id, project_id, path, message) VALUES (?1, ?2, ?3, ?4)",
                params![
                    Uuid::new_v4(),
                    project_id,
                    workspace_path,
                    "Missing name attribute"
                ],
            )?;
        }

        if package.version.is_none() {
            connection.execute(
                "INSERT INTO warnings (id, project_id, path, message) VALUES (?1, ?2, ?3, ?4)",
                params![
                    Uuid::new_v4(),
                    project_id,
                    workspace_path,
                    "Missing version attribute"
                ],
            )?;
        }

        let package_id = Uuid::new_v4();
        connection.execute(
            "INSERT INTO packages (id, project_id, path) VALUES (?1, ?2, ?3)",
            params![package_id, project_id, workspace_path],
        )?;

        let mut stmt = connection.prepare(
            "INSERT INTO dependencies (id, project_id, package_id, name, version, dev) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
        )?;

        if let Some(data) = package.dependencies {
            for (name, version) in data {
                stmt.execute(params![
                    Uuid::new_v4(),
                    project_id,
                    package_id,
                    name,
                    version,
                    false
                ])?;
            }
        }

        if let Some(data) = package.dev_dependencies {
            for (name, version) in data {
                stmt.execute(params![
                    Uuid::new_v4(),
                    project_id,
                    package_id,
                    name,
                    version,
                    true
                ])?;
            }
        }

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
        let inspector = PackageJsonInspector::new();
        assert!(inspector.supports_file(file.path()));

        file.close()?;
        Ok(())
    }

    #[test]
    fn requires_json_file_to_exist() {
        let path = Path::new("missing/package.json");
        let inspector = PackageJsonInspector::new();
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
