use crate::models::PackageJsonFile;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{named_params, params, Connection};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

pub struct ProjectInfo {
    pub id: i64,
    pub name: String,
    pub version: String,
    pub created_on: DateTime<Utc>,
    pub origin: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CodeWarning {
    pub path: String,
    pub message: String,
    pub url: String,
}

#[derive(Serialize)]
pub struct PackageFile {
    pub path: String,
    pub url: Option<String>,
}

#[derive(Serialize)]
pub struct PackageDependency {
    pub name: String,
    pub version: String,
    pub dev: bool,
    pub npm_url: String,
    pub package: String,
    pub url: String,
}

#[derive(Serialize)]
pub struct TestEntry {
    pub path: String,
    pub cases: i64,
    pub url: String,
}

#[derive(Serialize)]
pub struct AngularDirective {
    pub path: String,
    pub standalone: bool,
    pub url: Option<String>,
}

#[derive(Serialize)]
pub struct AngularFile {
    pub path: String,
    pub url: Option<String>,
}

pub fn create_connection(working_dir: &Path) -> Result<Connection> {
    let db_path = working_dir.join("birdview.db");
    let conn = Connection::open(db_path)?;

    conn.execute_batch(include_str!("assets/sql/schema.sql"))?;
    Ok(conn)
}

pub fn create_project(
    conn: &Connection,
    name: &String,
    version: &String,
    origin: Option<String>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO projects (name, version, created_on, origin) VALUES (?1, ?2, ?3, ?4)",
        params![name, version, Utc::now(), origin],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_project_by_id(conn: &Connection, project_id: i64) -> Result<ProjectInfo> {
    let project_info = conn.query_row(
        "SELECT name, version, created_on, origin FROM projects WHERE OID=:project_id",
        params![project_id],
        |row| {
            Ok(ProjectInfo {
                id: project_id,
                name: row.get(0)?,
                version: row.get(1)?,
                created_on: row.get(2)?,
                origin: row.get(3)?,
            })
        },
    )?;
    Ok(project_info)
}

pub fn create_ng_version(conn: &Connection, project_id: i64, version: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO angular (project_id, version) VALUES (?1, ?2)",
        params![project_id, version],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_ng_version(conn: &Connection, project_id: i64) -> rusqlite::Result<String> {
    conn.query_row(
        "SELECT version from angular WHERE project_id=:project_id",
        params![project_id],
        |row| row.get(0),
    )
}

pub fn create_warning(
    conn: &Connection,
    project_id: i64,
    path: &str,
    message: &str,
    url: &Option<String>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO warnings (project_id, path, message, url) VALUES (?1, ?2, ?3, ?4)",
        params![project_id, path, message, url],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn create_ng_module(
    conn: &Connection,
    project_id: i64,
    path: &str,
    url: &Option<String>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO ng_modules (project_id, path, url) VALUES (?1, ?2, ?3)",
        params![project_id, path, url],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn create_ng_component(
    conn: &Connection,
    project_id: i64,
    path: &str,
    standalone: bool,
    url: &Option<String>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO ng_components (project_id, path, standalone, url) VALUES (?1, ?2, ?3, ?4)",
        params![project_id, path, standalone, url],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn create_ng_directive(
    conn: &Connection,
    project_id: i64,
    path: &str,
    standalone: bool,
    url: &Option<String>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO ng_directives (project_id, path, standalone, url) VALUES (?1, ?2, ?3, ?4)",
        params![project_id, path, standalone, url],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn create_ng_service(
    conn: &Connection,
    project_id: i64,
    path: &str,
    url: &Option<String>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO ng_services (project_id, path, url) VALUES (?1, ?2, ?3)",
        params![project_id, path, url],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn create_ng_pipe(
    conn: &Connection,
    project_id: i64,
    path: &str,
    standalone: bool,
    url: &Option<String>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO ng_pipes (project_id, path, standalone, url) VALUES (?1, ?2, ?3, ?4)",
        params![project_id, path, standalone, url],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn create_ng_dialog(
    conn: &Connection,
    project_id: i64,
    path: &str,
    standalone: bool,
    url: &Option<String>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO ng_dialogs (project_id, path, standalone, url) VALUES (?1, ?2, ?3, ?4)",
        params![project_id, path, standalone, url],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn create_package(
    conn: &Connection,
    project_id: i64,
    path: &str,
    url: &Option<String>,
    package: &PackageJsonFile,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO packages (project_id, path, url) VALUES (?1, ?2, ?3)",
        params![project_id, path, url],
    )?;

    let package_id = conn.last_insert_rowid();

    let mut stmt = conn.prepare(
        "INSERT INTO dependencies (project_id, package_id, name, version, dev) VALUES (?1, ?2, ?3, ?4, ?5)"
    )?;

    if let Some(data) = &package.dependencies {
        for (name, version) in data {
            stmt.execute(params![project_id, package_id, name, version, false])?;
        }
    }

    if let Some(data) = &package.dev_dependencies {
        for (name, version) in data {
            stmt.execute(params![project_id, package_id, name, version, true])?;
        }
    }

    Ok(package_id)
}

pub fn create_unit_test(
    conn: &Connection,
    project_id: i64,
    path: &str,
    test_cases: Vec<String>,
    url: &Option<String>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO unit_tests (project_id, path, url) VALUES (?1, ?2, ?3)",
        params![project_id, path, url],
    )?;
    let test_id = conn.last_insert_rowid();
    let mut stmt = conn.prepare("INSERT INTO test_cases (test_id, name) VALUES (?1, ?2)")?;

    for name in test_cases {
        stmt.execute(params![test_id, name])?;
    }

    Ok(test_id)
}

pub fn create_e2e_test(
    conn: &Connection,
    project_id: i64,
    path: &str,
    test_cases: Vec<String>,
    url: &Option<String>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO e2e_tests (project_id, path, url) VALUES (?1, ?2, ?3)",
        params![project_id, path, url],
    )?;
    let test_id = conn.last_insert_rowid();
    let mut stmt = conn.prepare("INSERT INTO test_cases (test_id, name) VALUES (?1, ?2)")?;

    for name in test_cases {
        stmt.execute(params![test_id, name])?;
    }

    Ok(())
}

pub fn create_file_types(
    conn: &Connection,
    project_id: i64,
    types: &HashMap<String, i64>,
) -> Result<()> {
    let mut stmt =
        conn.prepare("INSERT INTO file_types (project_id, name, count) VALUES (?1, ?2, ?3)")?;

    for (key, value) in types {
        stmt.execute(params![project_id, key, value])?;
    }

    Ok(())
}

pub fn get_file_types(conn: &Connection, project_id: i64) -> Result<HashMap<String, i64>> {
    let mut stmt = conn.prepare(
        "SELECT name, count FROM file_types WHERE project_id=:project_id ORDER BY count DESC",
    )?;
    let file_types: HashMap<String, i64> = stmt
        .query_map(named_params! { ":project_id": project_id }, |x| {
            Ok((x.get(0)?, x.get(1)?))
        })?
        .flatten()
        .collect();
    Ok(file_types)
}

pub fn get_warnings(conn: &Connection, project_id: i64) -> Result<Vec<CodeWarning>> {
    let mut stmt =
        conn.prepare("SELECT path, message, url FROM warnings WHERE project_id=:project_id;")?;
    let rows = stmt
        .query_map(named_params! { ":project_id": project_id }, |row| {
            Ok(CodeWarning {
                path: row.get(0)?,
                message: row.get(1)?,
                url: row.get(2)?,
            })
        })?
        .filter_map(|entry| entry.ok())
        .collect();
    Ok(rows)
}

pub fn get_packages(conn: &Connection, project_id: i64) -> Result<Vec<PackageFile>> {
    let mut stmt = conn.prepare("SELECT path, url FROM packages WHERE project_id=:project_id")?;
    let rows = stmt
        .query_map(named_params! {":project_id": project_id}, |row| {
            Ok(PackageFile {
                path: row.get(0)?,
                url: row.get(1)?,
            })
        })?
        .filter_map(|entry| entry.ok())
        .collect();
    Ok(rows)
}

pub fn get_dependencies(conn: &Connection, project_id: i64) -> Result<Vec<PackageDependency>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT d.name, d.version, d.dev, p.path as package, p.url as url from dependencies d
        LEFT JOIN packages p on d.package_id = p.OID
        WHERE d.project_id=:project_id
        ORDER BY d.name
        "#,
    )?;

    let rows = stmt
        .query_map(named_params! {":project_id": project_id}, |row| {
            let name: String = row.get(0)?;
            let npm_url = format!("https://www.npmjs.com/package/{name}");

            Ok(PackageDependency {
                name,
                version: row.get(1)?,
                dev: row.get(2)?,
                npm_url,
                package: row.get(3)?,
                url: row.get(4)?,
            })
        })?
        .filter_map(|entry| entry.ok())
        .collect();
    Ok(rows)
}

pub fn get_unit_tests(conn: &Connection, project_id: i64) -> Result<Vec<TestEntry>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT ut.path, COUNT(DISTINCT tc.name) as cases, ut.url FROM unit_tests ut
          LEFT JOIN test_cases tc on ut.OID = tc.test_id
        WHERE ut.project_id=:project_id
        GROUP BY ut.path
    "#,
    )?;

    let rows = stmt
        .query_map(named_params! {":project_id": project_id}, |row| {
            Ok(TestEntry {
                path: row.get(0)?,
                cases: row.get(1)?,
                url: row.get(2)?,
            })
        })?
        .filter_map(|entry| entry.ok())
        .collect();
    Ok(rows)
}

pub fn get_e2e_tests(conn: &Connection, project_id: i64) -> Result<Vec<TestEntry>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT ut.path, COUNT(DISTINCT tc.name) as cases, ut.url FROM e2e_tests ut
          LEFT JOIN test_cases tc on ut.OID = tc.test_id
        WHERE ut.project_id=:project_id
        GROUP BY ut.path
    "#,
    )?;

    let rows = stmt
        .query_map(named_params! {":project_id": project_id}, |row| {
            Ok(TestEntry {
                path: row.get(0)?,
                cases: row.get(1)?,
                url: row.get(2)?,
            })
        })?
        .filter_map(|entry| entry.ok())
        .collect();
    Ok(rows)
}

pub fn get_ng_modules(conn: &Connection, project_id: i64) -> Result<Vec<AngularFile>> {
    let mut stmt =
        conn.prepare("SELECT path, url FROM ng_modules WHERE project_id=:project_id;")?;
    let rows = stmt
        .query_map(named_params! { ":project_id": project_id }, |row| {
            Ok(AngularFile {
                path: row.get(0)?,
                url: row.get(1)?,
            })
        })?
        .filter_map(|entry| entry.ok())
        .collect();
    Ok(rows)
}

pub fn get_ng_components(conn: &Connection, project_id: i64) -> Result<Vec<AngularDirective>> {
    let mut stmt = conn
        .prepare("SELECT path, standalone, url FROM ng_components WHERE project_id=:project_id;")?;
    let rows = stmt
        .query_map(named_params! { ":project_id": project_id }, |row| {
            Ok(AngularDirective {
                path: row.get(0)?,
                standalone: row.get(1)?,
                url: row.get(2)?,
            })
        })?
        .filter_map(|entry| entry.ok())
        .collect();
    Ok(rows)
}

pub fn get_ng_directives(conn: &Connection, project_id: i64) -> Result<Vec<AngularDirective>> {
    let mut stmt = conn
        .prepare("SELECT path, standalone, url FROM ng_directives WHERE project_id=:project_id;")?;
    let rows = stmt
        .query_map(named_params! { ":project_id": project_id }, |row| {
            Ok(AngularDirective {
                path: row.get(0)?,
                standalone: row.get(1)?,
                url: row.get(2)?,
            })
        })?
        .filter_map(|entry| entry.ok())
        .collect();
    Ok(rows)
}

pub fn get_ng_services(conn: &Connection, project_id: i64) -> Result<Vec<AngularFile>> {
    let mut stmt =
        conn.prepare("SELECT path, url FROM ng_services WHERE project_id=:project_id;")?;
    let rows = stmt
        .query_map(named_params! { ":project_id": project_id }, |row| {
            Ok(AngularFile {
                path: row.get(0)?,
                url: row.get(1)?,
            })
        })?
        .filter_map(|entry| entry.ok())
        .collect();
    Ok(rows)
}

pub fn get_ng_pipes(conn: &Connection, project_id: i64) -> Result<Vec<AngularDirective>> {
    let mut stmt =
        conn.prepare("SELECT path, standalone, url FROM ng_pipes WHERE project_id=:project_id;")?;
    let rows = stmt
        .query_map(named_params! { ":project_id": project_id }, |row| {
            Ok(AngularDirective {
                path: row.get(0)?,
                standalone: row.get(1)?,
                url: row.get(2)?,
            })
        })?
        .filter_map(|entry| entry.ok())
        .collect();
    Ok(rows)
}

pub fn get_ng_dialogs(conn: &Connection, project_id: i64) -> Result<Vec<AngularDirective>> {
    let mut stmt =
        conn.prepare("SELECT path, standalone, url FROM ng_dialogs WHERE project_id=:project_id;")?;
    let rows = stmt
        .query_map(named_params! { ":project_id": project_id }, |row| {
            Ok(AngularDirective {
                path: row.get(0)?,
                standalone: row.get(1)?,
                url: row.get(2)?,
            })
        })?
        .filter_map(|entry| entry.ok())
        .collect();
    Ok(rows)
}
