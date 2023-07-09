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

#[derive(PartialEq, Clone, Serialize)]
pub enum DependencyKind {
    Prod,
    Dev,
    Peer,
}

impl fmt::Display for DependencyKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                DependencyKind::Dev => "dev",
                DependencyKind::Peer => "peer",
                DependencyKind::Prod => "prod",
            }
        )
    }
}

impl ToSql for DependencyKind {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(self.to_string().into())
    }
}

impl FromSql for DependencyKind {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().map(|role| match role {
            "dev" => Ok(DependencyKind::Dev),
            "peer" => Ok(DependencyKind::Peer),
            "prod" => Ok(DependencyKind::Prod),
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

#[derive(Serialize)]
pub struct Snapshot {
    pub oid: i64,
    pub pid: i64,
    pub tag: Option<String>,
    pub created_on: DateTime<Utc>,
    pub sha: Option<String>,
    pub timestamp: DateTime<Utc>,
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
    pub oid: u64,
    pub sid: i64,
    pub path: String,
    pub message: String,
    pub url: String,
    pub tag: String,
    pub date: String,
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
    pub npm_url: String,
    pub package: String,
    pub url: String,
    pub kind: DependencyKind,
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

#[derive(Serialize, Debug)]
pub struct ProjectTag {
    pub pid: i64,
    pub sid: i64,
    pub name: String,
    pub date: String,
}

pub fn get_tags(conn: &Connection, pid: i64) -> Result<Vec<ProjectTag>> {
    let mut stmt = conn.prepare(
        "SELECT pid, OID AS sid, tag AS name, DATE(timestamp) AS date 
                FROM snapshots WHERE pid=:pid
                ORDER BY timestamp DESC",
    )?;
    let rows = stmt
        .query_map(named_params! {":pid": pid }, |row| {
            Ok(ProjectTag {
                pid: row.get(0)?,
                sid: row.get(1)?,
                name: row.get(2)?,
                date: row.get(3)?,
            })
        })?
        .filter_map(|entry| entry.ok())
        .collect();
    Ok(rows)
}

#[derive(Debug, Serialize)]
pub struct ProjectContributor {
    pub sid: i64,
    pub tag: String,
    pub name: String,
    pub commits: i64,
}

pub fn get_contributors(conn: &Connection, pid: i64, sid: i64) -> Result<Vec<ProjectContributor>> {
    let mut stmt = conn.prepare(
        "SELECT snapshots.OID AS sid, snapshots.tag, authors.name, authors.commits
                FROM authors
                LEFT JOIN snapshots ON snapshots.OID = authors.sid
                WHERE snapshots.pid=:pid AND snapshots.OID=:sid
                ORDER BY snapshots.OID, authors.commits DESC",
    )?;

    let rows = stmt
        .query_map(named_params! {":pid": pid, ":sid": sid }, |row| {
            Ok(ProjectContributor {
                sid: row.get(0)?,
                tag: row.get(1)?,
                name: row.get(2)?,
                commits: row.get(3)?,
            })
        })?
        .filter_map(|entry| entry.ok())
        .collect();
    Ok(rows)
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

pub fn get_project_snapshots(conn: &Connection, pid: i64) -> Result<Vec<Snapshot>> {
    let mut stmt = conn.prepare(
        "SELECT s.OID, s.pid, s.created_on, s.tag, s.sha, s.timestamp 
              FROM snapshots s
              WHERE s.pid=:pid",
    )?;

    let rows = stmt
        .query_map(named_params! {":pid": pid}, |row| {
            Ok(Snapshot {
                oid: row.get(0)?,
                pid: row.get(1)?,
                created_on: row.get(2)?,
                tag: row.get(3)?,
                sha: row.get(4)?,
                timestamp: row.get(5)?,
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
    tag: &String,
    project: &GitProject,
) -> Result<i64> {
    let sha = project.sha()?;
    let timestamp = project.timestamp()?;

    conn.execute(
        "INSERT INTO snapshots (pid, tag, sha, timestamp) VALUES (?1, ?2, ?3, ?4)",
        params![pid, tag, sha, timestamp],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_snapshot_by_id(conn: &Connection, oid: i64) -> rusqlite::Result<Snapshot> {
    conn.query_row(
        "SELECT s.pid, s.created_on, s.tag, s.sha, s.timestamp 
              FROM snapshots s WHERE s.OID=:oid",
        named_params! {":oid": oid },
        |row| {
            Ok(Snapshot {
                oid,
                pid: row.get(0)?,
                created_on: row.get(1)?,
                tag: row.get(2)?,
                sha: row.get(3)?,
                timestamp: row.get(4)?,
            })
        },
    )
}

pub fn get_snapshot_by_sha(conn: &Connection, sha: &str) -> Option<Snapshot> {
    let result = conn.query_row(
        "SELECT s.OID, s.pid, s.created_on, s.tag, s.sha, s.timestamp 
              FROM snapshots s WHERE s.sha=:sha",
        named_params! {":sha": sha },
        |row| {
            Ok(Snapshot {
                oid: row.get(0)?,
                pid: row.get(1)?,
                created_on: row.get(2)?,
                tag: row.get(3)?,
                sha: row.get(4)?,
                timestamp: row.get(5)?,
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
    let prod_deps = match &package.dependencies {
        Some(deps) => deps.len(),
        None => 0,
    };

    let dev_deps = match &package.dev_dependencies {
        Some(deps) => deps.len(),
        None => 0,
    };

    let peer_deps = match &package.peer_dependencies {
        Some(deps) => deps.len(),
        None => 0,
    };

    let name = match &package.name {
        Some(value) => value,
        None => "Unknown",
    };

    let version = match &package.version {
        Some(value) => value,
        None => "0.0.0",
    };

    conn.execute(
        "INSERT INTO packages (sid, path, name, version, url, prod_deps, dev_deps, peer_deps) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![sid, path, name, version, url, prod_deps, dev_deps, peer_deps],
    )?;

    let package_id = conn.last_insert_rowid();

    if prod_deps > 0 || dev_deps > 0 || peer_deps > 0 {
        let mut stmt = conn.prepare(
            "INSERT INTO dependencies (sid, package_id, name, version, kind) VALUES (?1, ?2, ?3, ?4, ?5)"
        )?;

        if let Some(data) = &package.dependencies {
            for (name, version) in data {
                stmt.execute(params![
                    sid,
                    package_id,
                    name,
                    version,
                    DependencyKind::Prod
                ])?;
            }
        }

        if let Some(data) = &package.dev_dependencies {
            for (name, version) in data {
                stmt.execute(params![sid, package_id, name, version, DependencyKind::Dev])?;
            }
        }

        if let Some(data) = &package.peer_dependencies {
            for (name, version) in data {
                stmt.execute(params![
                    sid,
                    package_id,
                    name,
                    version,
                    DependencyKind::Peer
                ])?;
            }
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

pub fn create_test(
    conn: &Connection,
    sid: i64,
    path: &str,
    test_cases: Vec<String>,
    url: &str,
    kind: TestKind,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO tests (sid, path, url, kind, cases) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![sid, path, url, kind, test_cases.len()],
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

pub fn get_project_warnings(conn: &Connection, pid: i64) -> Result<Vec<CodeWarning>> {
    let mut stmt = conn.prepare(
        "
        SELECT warnings.OID, warnings.sid, warnings.path, warnings.message, warnings.url,
                snapshots.tag, DATE(snapshots.timestamp) as time 
        FROM warnings
        LEFT JOIN snapshots ON snapshots.OID = warnings.sid
        WHERE snapshots.pid=:pid
        ORDER BY snapshots.timestamp",
    )?;

    let rows = stmt
        .query_map(named_params! { ":pid": pid }, |row| {
            Ok(CodeWarning {
                oid: row.get(0)?,
                sid: row.get(1)?,
                path: row.get(2)?,
                message: row.get(3)?,
                url: row.get(4)?,
                tag: row.get(5)?,
                date: row.get(6)?,
            })
        })?
        .filter_map(|entry| entry.ok())
        .collect();
    Ok(rows)
}

pub fn get_snapshot_warnings(conn: &Connection, sid: i64) -> Result<Vec<CodeWarning>> {
    let mut stmt = conn.prepare(
        "
        SELECT warnings.sid, warnings.path, warnings.message, warnings.url,
                snapshots.tag, DATE(snapshots.timestamp) as time 
        FROM warnings
        LEFT JOIN snapshots ON snapshots.OID = warnings.sid
        WHERE snapshots.OID=:sid
        ORDER BY snapshots.timestamp",
    )?;
    let rows = stmt
        .query_map(named_params! { ":sid": sid }, |row| {
            Ok(CodeWarning {
                oid: row.get(0)?,
                sid: row.get(1)?,
                path: row.get(2)?,
                message: row.get(3)?,
                url: row.get(4)?,
                tag: row.get(5)?,
                date: row.get(6)?,
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

#[derive(Serialize)]
pub struct ProjectDependencies {
    pub oid: i64,
    pub tag: String,
    pub date: String,
    pub dev_deps: i64,
    pub prod_deps: i64,
    pub peer_deps: i64,
}

pub fn get_project_dependencies(conn: &Connection, pid: i64) -> Result<Vec<ProjectDependencies>> {
    let mut stmt = conn.prepare(
        "
        SELECT snapshots.OID as sid, snapshots.tag, DATE(snapshots.timestamp) AS date,
            SUM(packages.dev_deps) AS dev_deps,
            SUM(packages.prod_deps) AS prod_deps,
            SUM(packages.peer_deps) AS peer_deps
        FROM snapshots
        LEFT JOIN packages ON packages.sid=snapshots.OID
        WHERE snapshots.pid = :pid
        GROUP BY snapshots.OID, date
        ORDER BY date",
    )?;

    let rows = stmt
        .query_map(named_params! {":pid": pid}, |row| {
            Ok(ProjectDependencies {
                oid: row.get(0)?,
                tag: row.get(1)?,
                date: row.get(2)?,
                dev_deps: row.get(3)?,
                prod_deps: row.get(4)?,
                peer_deps: row.get(5)?,
            })
        })?
        .filter_map(|entry| entry.ok())
        .collect();
    Ok(rows)
}

#[deprecated]
pub fn get_dependencies(conn: &Connection, sid: i64) -> Result<Vec<PackageDependency>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT d.name, d.version, d.kind, p.path as package, p.url as url from dependencies d
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
                kind: row.get(2)?,
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
    let mut stmt =
        conn.prepare("SELECT path, cases, url FROM tests WHERE sid=:sid AND kind=:kind")?;

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

#[derive(Serialize)]
pub struct TestInfo {
    pub oid: i64,
    pub tag: String,
    pub unit_tests: i64,
    pub unit_cases: i64,
    pub e2e_tests: i64,
    pub e2e_cases: i64,
}

pub fn get_tests_stats(conn: &Connection, pid: i64) -> Result<Vec<TestInfo>> {
    let mut stmt = conn.prepare(
        "
        SELECT
            snapshots.OID AS sid, snapshots.tag,
            SUM(case when tests.kind='unit' then 1 else 0 end) unit_tests,
            SUM(case when tests.kind='unit' then tests.cases else 0 end) unit_cases,
            SUM(case when tests.kind='e2e' then 1 else 0 end) e2e_tests,
            SUM(case when tests.kind='e2e' then tests.cases else 0 end) e2e_cases
        FROM snapshots
        LEFT JOIN tests ON tests.sid = snapshots.OID
        WHERE snapshots.pid = :pid
        GROUP BY snapshots.OID, snapshots.timestamp
        ORDER BY snapshots.timestamp",
    )?;

    let rows = stmt
        .query_map(named_params! {":pid": pid }, |row| {
            Ok(TestInfo {
                oid: row.get(0)?,
                tag: row.get(1)?,
                unit_tests: row.get(2)?,
                unit_cases: row.get(3)?,
                e2e_tests: row.get(4)?,
                e2e_cases: row.get(5)?,
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

pub fn generate_angular_metadata(
    conn: &Connection,
    sid: i64,
    version: &String,
) -> rusqlite::Result<()> {
    conn.execute_batch(
        format!(
            "
            BEGIN;
            INSERT INTO angular (sid, version, modules, components, directives, services, pipes, dialogs)
            SELECT
                sid,
                '{version}',
                SUM(case when kind='module' then 1 else 0 end) modules,
                SUM(case when kind='component' then 1 else 0 end) components,
                SUM(case when kind='directive' then 1 else 0 end) directives,
                SUM(case when kind='service' then 1 else 0 end) services,
                SUM(case when kind='pipe' then 1 else 0 end) pipes,
                SUM(case when kind='dialog' then 1 else 0 end) dialogs
            FROM ng_entities
            WHERE sid={sid};
            COMMIT;"
        )
        .as_str(),
    )
}

#[derive(Serialize)]
pub struct AngularMetadata {
    sid: i64,
    version: String,
    modules: i64,
    components: i64,
    directives: i64,
    services: i64,
    pipes: i64,
    dialogs: i64,
    tag: String,
    date: String,
}

pub fn get_angular_metadata(conn: &Connection, pid: i64) -> Result<Vec<AngularMetadata>> {
    let mut stmt = conn.prepare(
        "
        SELECT
            a.sid, a.version, a.modules, a.components, a.directives, a.services, a.pipes, a.dialogs,
            snapshots.tag, DATE(snapshots.timestamp) AS date
        FROM angular AS a
        LEFT JOIN snapshots ON snapshots.OID = a.sid
        WHERE snapshots.pid=:pid
        ORDER BY snapshots.timestamp",
    )?;

    let rows = stmt
        .query_map(named_params! { ":pid": pid }, |row| {
            Ok(AngularMetadata {
                sid: row.get(0)?,
                version: row.get(1)?,
                modules: row.get(2)?,
                components: row.get(3)?,
                directives: row.get(4)?,
                services: row.get(5)?,
                pipes: row.get(6)?,
                dialogs: row.get(7)?,
                tag: row.get(8)?,
                date: row.get(9)?,
            })
        })?
        .filter_map(|entry| entry.ok())
        .collect();

    Ok(rows)
}
