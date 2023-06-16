use anyhow::Result;
use rusqlite::{params, Connection};
use uuid::Uuid;

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
