pub mod config;
pub mod git;
pub mod inspectors;
pub mod models;

use crate::config::{Config, OutputFormat};
use crate::git::get_repository_info;
use crate::inspectors::*;
use crate::models::PackageJsonFile;
use chrono::Utc;
use ignore::WalkBuilder;
use rusqlite::{params, Connection};
use serde_json::{json, Map, Value};
use std::error::Error;
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

    let project_id = Uuid::new_v4();
    let connection = create_connection(&config.output_dir)?;

    let package = PackageJsonFile::from_file(package_json_path)?;

    if let Some(dependencies) = package.dependencies {
        if let Some(version) = dependencies.get("@angular/core") {
            connection.execute(
                "INSERT INTO angular (id, project_id, version) VALUES (?1, ?2, ?3)",
                params![Uuid::new_v4(), project_id, version],
            )?;
        }
    }

    connection
        .execute(
            "INSERT INTO projects (id, name, version, created_on) VALUES (?1, ?2, ?3, ?4)",
            params![project_id, package.name, package.version, Utc::now()],
        )
        .unwrap();

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
    if config.inspect_types {
        inspectors.push(Box::new(FileTypeInspector::new()));
    }

    if inspectors.is_empty() {
        println!("No inspectors defined.\nRun 'birdview inspect --help' for available options.");
        return Ok(());
    }

    let output = inspect(
        &config.working_dir,
        &connection,
        &project_id,
        inspectors,
        config.verbose,
    )?;

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

/// Performs the workspace analysis using the registered file inspectors
fn inspect(
    working_dir: &PathBuf,
    connection: &Connection,
    project_id: &Uuid,
    inspectors: Vec<Box<dyn FileInspector>>,
    verbose: bool,
) -> Result<Value, Box<dyn Error>> {
    if verbose {
        println!("{}", working_dir.display());
    }

    let mut map = Map::new();

    map.insert(
        "report_date".to_owned(),
        Value::String(Utc::now().to_string()),
    );

    let modules: Vec<&str> = inspectors
        .iter()
        .map(|inspector| inspector.get_module_name())
        .collect();

    if let Some(project) = get_project_info(working_dir, modules) {
        map.insert("project".to_owned(), project);
    }

    if let Some(repo) = get_repository_info(working_dir) {
        map.insert("git".to_owned(), json!(repo));
    }

    run_inspectors(
        working_dir,
        connection,
        project_id,
        inspectors,
        &mut map,
        verbose,
    );
    Ok(Value::Object(map))
}

fn run_inspectors(
    working_dir: &PathBuf,
    connection: &Connection,
    project_id: &Uuid,
    mut inspectors: Vec<Box<dyn FileInspector>>,
    map: &mut Map<String, Value>,
    verbose: bool,
) {
    for entry in WalkBuilder::new(working_dir)
        .build()
        .filter_map(|entry| entry.ok())
    {
        // let f_name = entry.file_name().to_string_lossy();
        let entry_path = entry.path();
        let mut processed = false;

        let options = FileInspectorOptions {
            working_dir: working_dir.to_owned(),
            path: entry_path.to_path_buf(),
            relative_path: entry_path.strip_prefix(working_dir).unwrap().to_owned(),
        };

        for inspector in inspectors.iter_mut() {
            if entry_path.is_file() && inspector.supports_file(entry_path) {
                inspector
                    .inspect_file(connection, project_id, &options, map)
                    .unwrap();
                processed = true;
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

    for inspector in inspectors.iter_mut() {
        inspector.finalize(connection, project_id, map).unwrap();
    }
}

fn get_project_info(working_dir: &Path, modules: Vec<&str>) -> Option<Value> {
    let package_json_path = working_dir.join("package.json");
    if package_json_path.exists() {
        let package = PackageJsonFile::from_file(&package_json_path).unwrap();

        return Some(json!({
            "name": package.name,
            "version": package.version,
            "modules": modules
        }));
    } else {
        println!("Warning: no package.json file found in the workspace");
    }

    None
}
