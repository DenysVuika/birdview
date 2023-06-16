use crate::models::PackageJsonFile;
use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection};
use std::collections::HashMap;

pub fn create_project(conn: &Connection, name: &String, version: &String) -> Result<i64> {
    conn.execute(
        "INSERT INTO projects (name, version, created_on) VALUES (?1, ?2, ?3)",
        params![name, version, Utc::now()],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn create_ng_version(conn: &Connection, project_id: i64, version: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO angular (project_id, version) VALUES (?1, ?2)",
        params![project_id, version],
    )?;
    Ok(conn.last_insert_rowid())
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
) -> Result<i64> {
    conn.execute(
        "INSERT INTO ng_directives (project_id, path, standalone) VALUES (?1, ?2, ?3)",
        params![project_id, path, standalone],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn create_ng_service(conn: &Connection, project_id: i64, path: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO ng_services (project_id, path) VALUES (?1, ?2)",
        params![project_id, path],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn create_ng_pipe(
    conn: &Connection,
    project_id: i64,
    path: &str,
    standalone: bool,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO ng_pipes (project_id, path, standalone) VALUES (?1, ?2, ?3)",
        params![project_id, path, standalone],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn create_ng_dialog(
    conn: &Connection,
    project_id: i64,
    path: &str,
    standalone: bool,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO ng_dialogs (project_id, path, standalone) VALUES (?1, ?2, ?3)",
        params![project_id, path, standalone],
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
) -> Result<i64> {
    conn.execute(
        "INSERT INTO unit_tests (project_id, path) VALUES (?1, ?2)",
        params![project_id, path],
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
) -> Result<()> {
    conn.execute(
        "INSERT INTO e2e_tests (project_id, path) VALUES (?1, ?2)",
        params![project_id, path],
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
