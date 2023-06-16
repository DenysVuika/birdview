pub mod config;
pub mod git;
pub mod inspectors;
pub mod models;

use crate::config::{Config, OutputFormat};
use crate::git::{get_repository_info, RepositoryInfo};
use crate::inspectors::*;
use crate::models::PackageJsonFile;
use chrono::Utc;
use ignore::WalkBuilder;
use rusqlite::{named_params, params, Connection};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn create_connection(working_dir: &Path) -> Result<Connection, Box<dyn Error>> {
    let db_path = working_dir.join("birdview.db");
    let conn = Connection::open(db_path)?;

    conn.execute_batch(include_str!("assets/sql/schema.sql"))?;
    Ok(conn)
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    let package_json_path = &config.working_dir.join("package.json");
    if !package_json_path.exists() {
        panic!("Cannot find package.json file");
    }

    let mut output = Map::new();
    output.insert(
        "report_date".to_owned(),
        Value::String(Utc::now().to_string()),
    );

    let connection = create_connection(&config.output_dir)?;
    let project_id = Uuid::new_v4();
    let package = PackageJsonFile::from_file(package_json_path)?;

    if let Some(dependencies) = package.dependencies {
        if let Some(version) = dependencies.get("@angular/core") {
            connection.execute(
                "INSERT INTO angular (id, project_id, version) VALUES (?1, ?2, ?3)",
                params![Uuid::new_v4(), project_id, version],
            )?;
            output.insert("angular_version".to_owned(), json!(version));
        }
    }

    output.insert(
        "project".to_owned(),
        json!({
            "name": package.name,
            "version": package.version
        }),
    );

    let repo = get_repository_info(&config.working_dir);

    if repo.is_some() {
        output.insert("git".to_owned(), json!(repo));
    }

    connection.execute(
        "INSERT INTO projects (id, name, version, created_on) VALUES (?1, ?2, ?3, ?4)",
        params![project_id, package.name, package.version, Utc::now()],
    )?;

    let mut inspectors: Vec<Box<dyn FileInspector>> = Vec::new();

    if config.inspect_packages {
        inspectors.push(Box::new(PackageJsonInspector::new()));
    }
    if config.inspect_tests {
        inspectors.push(Box::new(TestInspector::new()));
    }
    if config.inspect_angular {
        inspectors.push(Box::new(AngularInspector::new()));
    }

    if inspectors.is_empty() {
        println!("No inspectors defined.\nRun 'birdview inspect --help' for available options.");
        return Ok(());
    }

    run_inspectors(
        config,
        &connection,
        &project_id,
        inspectors,
        &mut output,
        config.verbose,
        &repo,
    )?;

    if let Ok(warnings) = get_warnings(&connection, &project_id) {
        output.insert("warnings".to_owned(), json!(warnings));
    }

    match get_dependencies(&connection, &project_id) {
        Ok(dependencies) => {
            output.insert("dependencies".to_owned(), json!(dependencies));
        }
        Err(err) => println!("{}", err),
    }

    match get_packages(&connection, &project_id) {
        Ok(packages) => {
            output.insert("packages".to_owned(), json!(packages));
        }
        Err(err) => println!("{}", err),
    }

    match get_angular_report(&connection, &project_id) {
        Ok(angular) => {
            output.entry("angular").or_insert(angular);
        }
        Err(err) => println!("{}", err),
    };

    match get_unit_tests(&connection, &project_id) {
        Ok(tests) => {
            output.entry("unit_tests").or_insert(json!(tests));
        }
        Err(err) => println!("{}", err),
    }

    match get_e2e_tests(&connection, &project_id) {
        Ok(tests) => {
            output.entry("e2e_tests").or_insert(json!(tests));
        }
        Err(err) => println!("{}", err),
    }

    let output_file_path = get_output_file(&config.output_dir, config.format).unwrap();
    let mut output_file = File::create(&output_file_path)?;
    let json_string = serde_json::to_string_pretty(&output)?;

    match &config.format {
        OutputFormat::Html => {
            let template = include_str!("assets/html/index.html");
            let data = format!("window.data = {};", json_string);
            let template = template.replace("// <birdview:DATA>", &data);

            write!(output_file, "{}", template)?;
            println!("Saved report to: {}", &output_file_path.display());

            if config.open {
                webbrowser::open(&output_file_path.display().to_string())?
            }
        }
        OutputFormat::Json => {
            write!(output_file, "{}", json_string)?;
            println!("Saved report to: {}", &output_file_path.display());
        }
    }

    println!("Inspection complete");
    Ok(())
}

fn get_output_file(output_dir: &Path, format: OutputFormat) -> Option<PathBuf> {
    let is_dir = output_dir.exists() && output_dir.is_dir();

    if is_dir {
        let extension = match format {
            OutputFormat::Html => "html",
            OutputFormat::Json => "json",
        };
        let now = chrono::offset::Local::now();
        let output_file =
            output_dir.join(format!("{}.{extension}", now.format("%Y-%m-%d_%H-%M-%S")));
        return Some(output_file);
    }

    None
}

fn run_inspectors(
    config: &Config,
    connection: &Connection,
    project_id: &Uuid,
    inspectors: Vec<Box<dyn FileInspector>>,
    map: &mut Map<String, Value>,
    verbose: bool,
    repo: &Option<RepositoryInfo>,
) -> Result<(), Box<dyn Error>> {
    let working_dir = &config.working_dir;
    let mut types: HashMap<String, i64> = HashMap::new();

    for entry in WalkBuilder::new(working_dir)
        .build()
        .filter_map(|entry| entry.ok())
    {
        // let f_name = entry.file_name().to_string_lossy();
        let entry_path = entry.path();
        let rel_path = entry_path.strip_prefix(working_dir)?.display().to_string();
        let mut processed = false;

        for inspector in inspectors.iter() {
            if entry_path.is_file() {
                if let Some(ext) = entry_path.extension().and_then(OsStr::to_str) {
                    let entry = types.entry(ext.to_owned()).or_insert(0);
                    *entry += 1;
                }

                if inspector.supports_file(entry_path) {
                    let url = match &repo {
                        None => None,
                        Some(repo) => {
                            let remote = &repo.remote;
                            let target = &repo.target;
                            let result = format!("{remote}/blob/{target}/{rel_path}");
                            Some(result)
                        }
                    };

                    let options = FileInspectorOptions {
                        project_id: project_id.to_owned(),
                        working_dir: working_dir.to_owned(),
                        path: entry_path.to_path_buf(),
                        relative_path: rel_path.to_owned(),
                        url,
                    };

                    inspector.inspect_file(connection, &options)?;
                    processed = true;
                }
            }
        }

        if verbose {
            println!(
                "├── {} {}",
                if processed { '✅' } else { '🔎' },
                entry_path.strip_prefix(working_dir).unwrap().display()
            );
        }
    }

    if config.inspect_types && !types.is_empty() {
        save_file_types(connection, project_id, &types)?;
        map.entry("types").or_insert(json!(types));
    }

    Ok(())
}

fn save_file_types(
    connection: &Connection,
    project_id: &Uuid,
    types: &HashMap<String, i64>,
) -> Result<(), Box<dyn Error>> {
    let mut stmt = connection
        .prepare("INSERT INTO file_types (project_id, name, count) VALUES (?1, ?2, ?3)")?;

    for (key, value) in types {
        stmt.execute(params![project_id, key, value])?;
    }

    Ok(())
}

#[derive(Debug, Serialize)]
struct CodeWarning {
    path: String,
    message: String,
    url: String,
}

fn get_warnings(
    connection: &Connection,
    project_id: &Uuid,
) -> Result<Vec<CodeWarning>, Box<dyn Error>> {
    let mut stmt = connection
        .prepare("SELECT path, message, url FROM warnings WHERE project_id=:project_id;")?;
    let rows = stmt.query_map(named_params! { ":project_id": project_id }, |row| {
        Ok(CodeWarning {
            path: row.get(0)?,
            message: row.get(1)?,
            url: row.get(2)?,
        })
    })?;
    let warnings: Vec<CodeWarning> = rows.filter_map(|entry| entry.ok()).collect();
    Ok(warnings)
}

#[derive(Serialize)]
struct PackageFile {
    path: String,
    url: Option<String>,
}

#[derive(Serialize)]
struct PackageDependency {
    name: String,
    version: String,
    dev: bool,
    npm_url: String,
    package: String,
    url: String,
}

fn get_packages(
    connection: &Connection,
    project_id: &Uuid,
) -> Result<Vec<PackageFile>, Box<dyn Error>> {
    let mut stmt =
        connection.prepare("SELECT path, url FROM packages WHERE project_id=:project_id")?;
    let rows = stmt.query_map(named_params! {":project_id": project_id}, |row| {
        Ok(PackageFile {
            path: row.get(0)?,
            url: row.get(1)?,
        })
    })?;

    let entries: Vec<PackageFile> = rows.filter_map(|entry| entry.ok()).collect();
    Ok(entries)
}

fn get_dependencies(
    connection: &Connection,
    project_id: &Uuid,
) -> Result<Vec<PackageDependency>, Box<dyn Error>> {
    let mut stmt = connection.prepare(
        r#"
        SELECT d.name, d.version, d.dev, d.npm_url, p.path as package, p.url as url from dependencies d
        LEFT JOIN packages p on d.package_id = p.id
        WHERE d.project_id=:project_id
        ORDER BY d.name
        "#,
    )?;

    let rows = stmt.query_map(named_params! {":project_id": project_id}, |row| {
        Ok(PackageDependency {
            name: row.get(0)?,
            version: row.get(1)?,
            dev: row.get(2)?,
            npm_url: row.get(3)?,
            package: row.get(4)?,
            url: row.get(5)?,
        })
    })?;

    let entries: Vec<PackageDependency> = rows.filter_map(|entry| entry.ok()).collect();
    Ok(entries)
}

#[derive(Serialize)]
pub struct AngularDirective {
    path: String,
    standalone: bool,
}

#[derive(Serialize)]
pub struct AngularFile {
    path: String,
}

// todo: return urls
fn get_angular_report(connection: &Connection, project_id: &Uuid) -> Result<Value, Box<dyn Error>> {
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

#[derive(Serialize)]
struct TestEntry {
    path: String,
    cases: i64,
}

fn get_unit_tests(
    connection: &Connection,
    project_id: &Uuid,
) -> Result<Vec<TestEntry>, Box<dyn Error>> {
    let mut stmt = connection.prepare(
        r#"
        SELECT ut.path, COUNT(DISTINCT tc.name) as cases FROM unit_tests ut
          LEFT JOIN test_cases tc on ut.id = tc.test_id
        WHERE ut.project_id=:project_id
        GROUP BY ut.path
    "#,
    )?;

    let rows = stmt.query_map(named_params! {":project_id": project_id}, |row| {
        Ok(TestEntry {
            path: row.get(0)?,
            cases: row.get(1)?,
        })
    })?;

    let entries: Vec<TestEntry> = rows.filter_map(|entry| entry.ok()).collect();
    Ok(entries)
}

fn get_e2e_tests(
    connection: &Connection,
    project_id: &Uuid,
) -> Result<Vec<TestEntry>, Box<dyn Error>> {
    let mut stmt = connection.prepare(
        r#"
        SELECT ut.path, COUNT(DISTINCT tc.name) as cases FROM e2e_tests ut
          LEFT JOIN test_cases tc on ut.id = tc.test_id
        WHERE ut.project_id=:project_id
        GROUP BY ut.path
    "#,
    )?;

    let rows = stmt.query_map(named_params! {":project_id": project_id}, |row| {
        Ok(TestEntry {
            path: row.get(0)?,
            cases: row.get(1)?,
        })
    })?;

    let entries: Vec<TestEntry> = rows.filter_map(|entry| entry.ok()).collect();
    Ok(entries)
}
