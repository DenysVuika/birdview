use crate::models::PackageJsonFile;
use anyhow::Result;
use rusqlite::{params, Connection};
use uuid::Uuid;

pub fn create_warning(
    conn: &Connection,
    project_id: &Uuid,
    path: &str,
    message: &str,
    url: &Option<String>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO warnings (project_id, path, message, url) VALUES (?1, ?2, ?3, ?4)",
        params![project_id, path, message, url],
    )?;
    Ok(())
}

pub fn create_ng_module(
    conn: &Connection,
    project_id: &Uuid,
    path: &str,
    url: &Option<String>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO ng_modules (project_id, path, url) VALUES (?1, ?2, ?3)",
        params![project_id, path, url],
    )?;

    Ok(())
}

pub fn create_ng_component(
    conn: &Connection,
    project_id: &Uuid,
    path: &str,
    standalone: bool,
    url: &Option<String>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO ng_components (project_id, path, standalone, url) VALUES (?1, ?2, ?3, ?4)",
        params![project_id, path, standalone, url],
    )?;
    Ok(())
}

pub fn create_ng_directive(
    conn: &Connection,
    project_id: &Uuid,
    path: &str,
    standalone: bool,
) -> Result<()> {
    conn.execute(
        "INSERT INTO ng_directives (project_id, path, standalone) VALUES (?1, ?2, ?3)",
        params![project_id, path, standalone],
    )?;
    Ok(())
}

pub fn create_ng_service(conn: &Connection, project_id: &Uuid, path: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO ng_services (project_id, path) VALUES (?1, ?2)",
        params![project_id, path],
    )?;
    Ok(())
}

pub fn create_ng_pipe(
    conn: &Connection,
    project_id: &Uuid,
    path: &str,
    standalone: bool,
) -> Result<()> {
    conn.execute(
        "INSERT INTO ng_pipes (project_id, path, standalone) VALUES (?1, ?2, ?3)",
        params![project_id, path, standalone],
    )?;
    Ok(())
}

pub fn create_ng_dialog(
    conn: &Connection,
    project_id: &Uuid,
    path: &str,
    standalone: bool,
) -> Result<()> {
    conn.execute(
        "INSERT INTO ng_dialogs (project_id, path, standalone) VALUES (?1, ?2, ?3)",
        params![project_id, path, standalone],
    )?;
    Ok(())
}

pub fn create_package(
    conn: &Connection,
    project_id: &Uuid,
    path: &str,
    url: &Option<String>,
    package: &PackageJsonFile,
) -> Result<Uuid> {
    let package_id = Uuid::new_v4();

    conn.execute(
        "INSERT INTO packages (id, project_id, path, url) VALUES (?1, ?2, ?3, ?4)",
        params![package_id, project_id, path, url],
    )?;

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
    project_id: &Uuid,
    path: &str,
    test_cases: Vec<String>,
) -> Result<()> {
    let test_id = Uuid::new_v4();

    conn.execute(
        "INSERT INTO unit_tests (id, project_id, path) VALUES (?1, ?2, ?3)",
        params![test_id, project_id, path],
    )?;

    let mut stmt = conn.prepare("INSERT INTO test_cases (test_id, name) VALUES (?1, ?2)")?;

    for name in test_cases {
        stmt.execute(params![test_id, name])?;
    }

    Ok(())
}

pub fn create_e2e_test(
    conn: &Connection,
    project_id: &Uuid,
    path: &str,
    test_cases: Vec<String>,
) -> Result<()> {
    let test_id = Uuid::new_v4();

    conn.execute(
        "INSERT INTO e2e_tests (id, project_id, path) VALUES (?1, ?2, ?3)",
        params![test_id, project_id, path],
    )?;

    let mut stmt = conn.prepare("INSERT INTO test_cases (test_id, name) VALUES (?1, ?2)")?;

    for name in test_cases {
        stmt.execute(params![test_id, name])?;
    }

    Ok(())
}
