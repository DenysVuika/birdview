use crate::git::{AuthorInfo, GitProject};
use crate::models::PackageJsonFile;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef};
use rusqlite::{named_params, params, Connection, ToSql};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;

#[derive(PartialEq, Clone)]
pub enum TestKind {
    Unit,
    EndToEnd,
}

impl fmt::Display for TestKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                TestKind::Unit => "unit",
                TestKind::EndToEnd => "e2e",
            }
        )
    }
}

impl ToSql for TestKind {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(self.to_string().into())
    }
}

impl FromSql for TestKind {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().map(|role| match role {
            "unit" => Ok(TestKind::Unit),
            "e2e" => Ok(TestKind::EndToEnd),
            _ => Err(FromSqlError::Other("Invalid role found in db".into())),
        })?
    }
}

/// Angular entity kind
#[derive(PartialEq, Clone)]
pub enum NgKind {
    Unknown,
    Module,
    Component,
    Directive,
    Service,
    Pipe,
    Dialog,
}

impl fmt::Display for NgKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                NgKind::Unknown => "unknown",
                NgKind::Module => "module",
                NgKind::Component => "component",
                NgKind::Directive => "directive",
                NgKind::Service => "service",
                NgKind::Pipe => "pipe",
                NgKind::Dialog => "dialog",
            }
        )
    }
}

impl ToSql for NgKind {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(self.to_string().into())
    }
}

impl FromSql for NgKind {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().map(|role| match role {
            "module" => Ok(NgKind::Module),
            "component" => Ok(NgKind::Component),
            "directive" => Ok(NgKind::Directive),
            "service" => Ok(NgKind::Service),
            "pipe" => Ok(NgKind::Pipe),
            "dialog" => Ok(NgKind::Dialog),
            "unknown" => Ok(NgKind::Unknown),
            _ => Err(FromSqlError::Other("Invalid role found in db".into())),
        })?
    }
}

pub struct Snapshot {
    pub oid: i64,
    pub pid: i64,
    pub created_on: DateTime<Utc>,
    pub branch: Option<String>,
    pub sha: Option<String>,
}

#[derive(Serialize)]
pub struct ProjectInfo {
    pub id: i64,
    pub name: String,
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
pub struct NgEntity {
    pub path: String,
    pub standalone: bool,
    pub url: Option<String>,
}

pub fn create_connection(working_dir: &Path) -> Result<Connection> {
    let db_path = working_dir.join("birdview.db");
    let conn = Connection::open(db_path)?;

    conn.execute_batch(include_str!("assets/sql/schema.sql"))?;
    Ok(conn)
}

pub fn create_project(conn: &Connection, name: &String, origin: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO projects (name, origin) VALUES (?1, ?2)",
        params![name, origin],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Create a list of tags for a given project
pub fn create_tags(conn: &Connection, pid: i64, tags: &Vec<String>) -> Result<()> {
    let mut stmt = conn.prepare("INSERT INTO tags (pid, name) VALUES (?1, ?2)")?;

    for name in tags {
        stmt.execute(params![pid, name])?;
    }
    Ok(())
}

pub fn create_tag(conn: &Connection, pid: i64, name: &str) -> Result<i64> {
    let mut stmt = conn.prepare("INSERT INTO tags (pid, name) VALUES (?1, ?2)")?;
    stmt.execute(params![pid, name])?;
    Ok(conn.last_insert_rowid())
}

pub fn get_projects(conn: &Connection) -> Result<Vec<ProjectInfo>> {
    let mut stmt = conn.prepare("SELECT OID, name, created_on, origin FROM projects")?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ProjectInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                created_on: row.get(2)?,
                origin: row.get(3)?,
            })
        })?
        .filter_map(|entry| entry.ok())
        .collect();
    Ok(rows)
}

pub fn get_project_by_name(conn: &Connection, name: &str) -> Result<ProjectInfo> {
    let project_info = conn.query_row(
        "SELECT OID, name, created_on, origin FROM projects WHERE name=:name",
        params![name],
        |row| {
            Ok(ProjectInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                created_on: row.get(2)?,
                origin: row.get(3)?,
            })
        },
    )?;
    Ok(project_info)
}

pub fn get_project_by_snapshot(conn: &Connection, sid: i64) -> Result<ProjectInfo> {
    let project_info = conn.query_row(
        "SELECT p.OID, p.name, p.created_on, p.origin FROM snapshots s JOIN projects p ON p.OID = s.pid WHERE s.OID=:sid",
        params![sid],
        |row| {
            Ok(ProjectInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                created_on: row.get(2)?,
                origin: row.get(3)?,
            })
        },
    )?;
    Ok(project_info)
}

pub fn create_snapshot(
    conn: &Connection,
    pid: i64,
    tag_id: i64,
    project: &GitProject,
) -> Result<i64> {
    let sha = project.sha()?;
    let timestamp = project.timestamp()?;

    conn.execute(
        "INSERT INTO snapshots (pid, tag_id, sha, timestamp) VALUES (?1, ?2, ?3, ?4)",
        params![pid, tag_id, sha, timestamp],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_snapshot_by_id(conn: &Connection, oid: i64) -> rusqlite::Result<Snapshot> {
    conn.query_row(
        "SELECT s.pid, s.created_on, t.name AS branch, s.sha FROM snapshots s
                JOIN tags t on s.tag_id = t.OID
                WHERE s.OID=:oid",
        named_params! {":oid": oid },
        |row| {
            Ok(Snapshot {
                oid,
                pid: row.get(0)?,
                created_on: row.get(1)?,
                branch: row.get(2)?,
                sha: row.get(3)?,
            })
        },
    )
}

pub fn get_snapshot_by_sha(conn: &Connection, sha: &str) -> Option<Snapshot> {
    let result = conn.query_row(
        "SELECT s.OID, s.pid, s.created_on, t.name AS branch, s.sha FROM snapshots s
                JOIN tags t on s.tag_id = t.OID
                WHERE s.sha=:sha",
        named_params! {":sha": sha },
        |row| {
            Ok(Snapshot {
                oid: row.get(0)?,
                pid: row.get(1)?,
                created_on: row.get(2)?,
                branch: row.get(3)?,
                sha: row.get(4)?,
            })
        },
    );

    match result {
        Ok(snapshot) => Some(snapshot),
        Err(err) => {
            log::error!("Snapshot not found: {}", err);
            None
        }
    }
}

pub fn has_snapshot(conn: &Connection, sha: &str) -> bool {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(OID) from snapshots WHERE sha=:sha",
            named_params! {":sha": sha},
            |row| row.get(0),
        )
        .unwrap_or(0);
    count > 0
}

pub fn create_ng_version(conn: &Connection, sid: i64, version: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO ng_version (sid, version) VALUES (?1, ?2)",
        params![sid, version],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_ng_version(conn: &Connection, sid: i64) -> rusqlite::Result<String> {
    conn.query_row(
        "SELECT version from ng_version WHERE sid=:sid",
        params![sid],
        |row| row.get(0),
    )
}

pub fn create_warning(
    conn: &Connection,
    sid: i64,
    path: &str,
    message: &str,
    url: &str,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO warnings (sid, path, message, url) VALUES (?1, ?2, ?3, ?4)",
        params![sid, path, message, url],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn create_ng_entity(
    conn: &Connection,
    sid: i64,
    kind: NgKind,
    path: &str,
    url: &str,
    standalone: bool,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO ng_entities (sid, kind, path, url, standalone) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![sid, kind, path, url, standalone],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn create_package(
    conn: &Connection,
    sid: i64,
    path: &str,
    url: &str,
    package: &PackageJsonFile,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO packages (sid, path, url) VALUES (?1, ?2, ?3)",
        params![sid, path, url],
    )?;

    let package_id = conn.last_insert_rowid();

    let mut stmt = conn.prepare(
        "INSERT INTO dependencies (sid, package_id, name, version, dev) VALUES (?1, ?2, ?3, ?4, ?5)"
    )?;

    if let Some(data) = &package.dependencies {
        for (name, version) in data {
            stmt.execute(params![sid, package_id, name, version, false])?;
        }
    }

    if let Some(data) = &package.dev_dependencies {
        for (name, version) in data {
            stmt.execute(params![sid, package_id, name, version, true])?;
        }
    }

    Ok(package_id)
}

pub fn create_authors(conn: &Connection, sid: i64, authors: &Vec<AuthorInfo>) -> Result<()> {
    let mut stmt = conn.prepare("INSERT INTO authors (sid, name, commits) VALUES (?1, ?2, ?3)")?;

    for author in authors {
        stmt.execute(params![sid, author.name, author.commits])?;
    }
    Ok(())
}

pub fn get_authors(conn: &Connection, sid: i64) -> Result<Vec<AuthorInfo>> {
    let mut stmt = conn.prepare("SELECT name, commits FROM authors WHERE sid=:sid;")?;
    let rows = stmt
        .query_map(named_params! { ":sid": sid }, |row| {
            Ok(AuthorInfo {
                name: row.get(0)?,
                commits: row.get(1)?,
            })
        })?
        .filter_map(|entry| entry.ok())
        .collect();
    Ok(rows)
}

pub fn create_test(
    conn: &Connection,
    sid: i64,
    path: &str,
    test_cases: Vec<String>,
    url: &str,
    kind: TestKind,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO tests (sid, path, url, kind) VALUES (?1, ?2, ?3, ?4)",
        params![sid, path, url, kind],
    )?;
    let test_id = conn.last_insert_rowid();
    let mut stmt = conn.prepare("INSERT INTO test_cases (test_id, name) VALUES (?1, ?2)")?;

    for name in test_cases {
        stmt.execute(params![test_id, name])?;
    }

    Ok(test_id)
}

pub fn create_file_types(conn: &Connection, sid: i64, types: &HashMap<String, i64>) -> Result<()> {
    let mut stmt = conn.prepare("INSERT INTO file_types (sid, name, count) VALUES (?1, ?2, ?3)")?;

    for (key, value) in types {
        stmt.execute(params![sid, key, value])?;
    }

    Ok(())
}

pub fn get_file_types(conn: &Connection, sid: i64) -> Result<HashMap<String, i64>> {
    let mut stmt =
        conn.prepare("SELECT name, count FROM file_types WHERE sid=:sid ORDER BY count DESC")?;
    let file_types: HashMap<String, i64> = stmt
        .query_map(named_params! { ":sid": sid }, |x| {
            Ok((x.get(0)?, x.get(1)?))
        })?
        .flatten()
        .collect();
    Ok(file_types)
}

pub fn get_warnings(conn: &Connection, sid: i64) -> Result<Vec<CodeWarning>> {
    let mut stmt = conn.prepare("SELECT path, message, url FROM warnings WHERE sid=:sid;")?;
    let rows = stmt
        .query_map(named_params! { ":sid": sid }, |row| {
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

pub fn get_packages(conn: &Connection, sid: i64) -> Result<Vec<PackageFile>> {
    let mut stmt = conn.prepare("SELECT path, url FROM packages WHERE sid=:sid")?;
    let rows = stmt
        .query_map(named_params! {":sid": sid}, |row| {
            Ok(PackageFile {
                path: row.get(0)?,
                url: row.get(1)?,
            })
        })?
        .filter_map(|entry| entry.ok())
        .collect();
    Ok(rows)
}

pub fn get_dependencies(conn: &Connection, sid: i64) -> Result<Vec<PackageDependency>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT d.name, d.version, d.dev, p.path as package, p.url as url from dependencies d
        LEFT JOIN packages p on d.package_id = p.OID
        WHERE d.sid=:sid
        ORDER BY d.name
        "#,
    )?;

    let rows = stmt
        .query_map(named_params! {":sid": sid}, |row| {
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

pub fn get_tests(conn: &Connection, sid: i64, kind: TestKind) -> Result<Vec<TestEntry>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT t.path, COUNT(DISTINCT tc.name) as cases, t.url FROM tests t
          LEFT JOIN test_cases tc on t.OID = tc.test_id
        WHERE t.sid=:sid AND t.kind=:kind
        GROUP BY t.path
    "#,
    )?;

    let rows = stmt
        .query_map(named_params! {":sid": sid, ":kind": kind}, |row| {
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

pub fn get_ng_entities(conn: &Connection, sid: i64, kind: NgKind) -> Result<Vec<NgEntity>> {
    let mut stmt = conn
        .prepare("SELECT path, url, standalone FROM ng_entities WHERE sid=:sid AND kind=:kind;")?;
    let rows = stmt
        .query_map(named_params! { ":sid": sid, ":kind": kind }, |row| {
            Ok(NgEntity {
                path: row.get(0)?,
                url: row.get(1)?,
                standalone: row.get(2)?,
            })
        })?
        .filter_map(|entry| entry.ok())
        .collect();
    Ok(rows)
}
