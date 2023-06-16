pub mod config;
pub mod db;
pub mod git;
pub mod inspectors;
pub mod models;

use crate::config::{Config, OutputFormat};
use crate::git::{get_repository_info, RepositoryInfo};
use crate::inspectors::*;
use crate::models::PackageJsonFile;
use anyhow::Result;
use ignore::WalkBuilder;
use rusqlite::{named_params, Connection};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

fn create_connection(working_dir: &Path) -> Result<Connection> {
    let db_path = working_dir.join("birdview.db");
    let conn = Connection::open(db_path)?;

    conn.execute_batch(include_str!("assets/sql/schema.sql"))?;
    Ok(conn)
}

pub fn run(config: &Config) -> Result<()> {
    let package_json_path = &config.working_dir.join("package.json");
    if !package_json_path.exists() {
        panic!("Cannot find package.json file");
    }

    let mut output = Map::new();
    // output.insert(
    //     "report_date".to_owned(),
    //     Value::String(Utc::now().to_string()),
    // );

    let repo = get_repository_info(&config.working_dir);
    let connection = create_connection(&config.output_dir)?;
    let package = PackageJsonFile::from_file(package_json_path)?;

    let name = package.name.unwrap();
    let version = package.version.unwrap();
    let project_id = db::create_project(&connection, &name, &version, None)?;

    if repo.is_some() {
        output.insert("git".to_owned(), json!(repo));
    }

    if let Some(dependencies) = package.dependencies {
        if let Some(version) = dependencies.get("@angular/core") {
            db::create_ng_version(&connection, project_id, version)?;
        }
    }

    let inspectors: Vec<Box<dyn FileInspector>> = vec![
        Box::new(PackageJsonInspector {}),
        Box::new(TestInspector {}),
        Box::new(AngularInspector {}),
    ];

    run_inspectors(
        config,
        &connection,
        project_id,
        inspectors,
        &mut output,
        config.verbose,
        &repo,
    )?;

    generate_report(&connection, project_id, &mut output)?;

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

fn generate_report(
    conn: &Connection,
    project_id: i64,
    output: &mut Map<String, Value>,
) -> Result<()> {
    let project = db::get_project_by_id(conn, project_id)?;

    let ng_version = db::get_ng_version(conn, project_id)?;
    output.insert("angular_version".to_owned(), json!(ng_version));

    output.insert(
        "project".to_owned(),
        json!({
            "name": project.name,
            "version": project.version,
            "created_on": project.created_on,
            "origin": project.origin
        }),
    );

    let warnings = get_warnings(conn, project_id).unwrap_or(vec![]);
    output.insert("warnings".to_owned(), json!(warnings));

    match get_dependencies(conn, project_id) {
        Ok(dependencies) => {
            output.insert("dependencies".to_owned(), json!(dependencies));
        }
        Err(err) => println!("{}", err),
    }

    match get_packages(conn, project_id) {
        Ok(packages) => {
            output.insert("packages".to_owned(), json!(packages));
        }
        Err(err) => println!("{}", err),
    }

    match get_angular_report(conn, project_id) {
        Ok(angular) => {
            output.entry("angular").or_insert(angular);
        }
        Err(err) => println!("{}", err),
    };

    match get_unit_tests(conn, project_id) {
        Ok(tests) => {
            output.entry("unit_tests").or_insert(json!(tests));
        }
        Err(err) => println!("{}", err),
    }

    match get_e2e_tests(conn, project_id) {
        Ok(tests) => {
            output.entry("e2e_tests").or_insert(json!(tests));
        }
        Err(err) => println!("{}", err),
    }

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
    project_id: i64,
    inspectors: Vec<Box<dyn FileInspector>>,
    map: &mut Map<String, Value>,
    verbose: bool,
    repo: &Option<RepositoryInfo>,
) -> Result<()> {
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
                            let remote = &repo.remote_url;
                            let target = &repo.sha;
                            let result = format!("{remote}/blob/{target}/{rel_path}");
                            Some(result)
                        }
                    };

                    let options = FileInspectorOptions {
                        project_id,
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

    if !types.is_empty() {
        db::create_file_types(connection, project_id, &types)?;
        map.entry("types").or_insert(json!(types));
    }

    Ok(())
}

#[derive(Debug, Serialize)]
struct CodeWarning {
    path: String,
    message: String,
    url: String,
}

fn get_warnings(conn: &Connection, project_id: i64) -> Result<Vec<CodeWarning>> {
    let mut stmt =
        conn.prepare("SELECT path, message, url FROM warnings WHERE project_id=:project_id;")?;
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

fn get_packages(conn: &Connection, project_id: i64) -> Result<Vec<PackageFile>> {
    let mut stmt = conn.prepare("SELECT path, url FROM packages WHERE project_id=:project_id")?;
    let rows = stmt.query_map(named_params! {":project_id": project_id}, |row| {
        Ok(PackageFile {
            path: row.get(0)?,
            url: row.get(1)?,
        })
    })?;

    let entries: Vec<PackageFile> = rows.filter_map(|entry| entry.ok()).collect();
    Ok(entries)
}

fn get_dependencies(conn: &Connection, project_id: i64) -> Result<Vec<PackageDependency>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT d.name, d.version, d.dev, p.path as package, p.url as url from dependencies d
        LEFT JOIN packages p on d.package_id = p.OID
        WHERE d.project_id=:project_id
        ORDER BY d.name
        "#,
    )?;

    let rows = stmt.query_map(named_params! {":project_id": project_id}, |row| {
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
fn get_angular_report(conn: &Connection, project_id: i64) -> Result<Value> {
    let mut stmt = conn.prepare("SELECT path FROM ng_modules WHERE project_id=:project_id;")?;
    let rows = stmt.query_map(named_params! { ":project_id": project_id }, |row| {
        Ok(AngularFile {
            path: row.get(0).unwrap(),
        })
    })?;
    let modules: Vec<AngularFile> = rows.filter_map(|entry| entry.ok()).collect();

    let mut stmt =
        conn.prepare("SELECT path, standalone FROM ng_components WHERE project_id=:project_id;")?;
    let rows = stmt.query_map(named_params! { ":project_id": project_id }, |row| {
        Ok(AngularDirective {
            path: row.get(0)?,
            standalone: row.get(1)?,
        })
    })?;
    let components: Vec<AngularDirective> = rows.filter_map(|entry| entry.ok()).collect();

    let mut stmt =
        conn.prepare("SELECT path, standalone FROM ng_directives WHERE project_id=:project_id;")?;
    let rows = stmt.query_map(named_params! { ":project_id": project_id }, |row| {
        Ok(AngularDirective {
            path: row.get(0)?,
            standalone: row.get(1)?,
        })
    })?;
    let directives: Vec<AngularDirective> = rows.filter_map(|entry| entry.ok()).collect();

    let mut stmt = conn.prepare("SELECT path FROM ng_services WHERE project_id=:project_id;")?;
    let rows = stmt.query_map(named_params! { ":project_id": project_id }, |row| {
        Ok(AngularFile {
            path: row.get(0).unwrap(),
        })
    })?;
    let services: Vec<AngularFile> = rows.filter_map(|entry| entry.ok()).collect();

    let mut stmt =
        conn.prepare("SELECT path, standalone FROM ng_pipes WHERE project_id=:project_id;")?;
    let rows = stmt.query_map(named_params! { ":project_id": project_id }, |row| {
        Ok(AngularDirective {
            path: row.get(0)?,
            standalone: row.get(1)?,
        })
    })?;
    let pipes: Vec<AngularDirective> = rows.filter_map(|entry| entry.ok()).collect();

    let mut stmt =
        conn.prepare("SELECT path, standalone FROM ng_dialogs WHERE project_id=:project_id;")?;
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
    url: String,
}

fn get_unit_tests(conn: &Connection, project_id: i64) -> Result<Vec<TestEntry>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT ut.path, COUNT(DISTINCT tc.name) as cases FROM unit_tests ut
          LEFT JOIN test_cases tc on ut.OID = tc.test_id
        WHERE ut.project_id=:project_id
        GROUP BY ut.path
    "#,
    )?;

    let rows = stmt.query_map(named_params! {":project_id": project_id}, |row| {
        Ok(TestEntry {
            path: row.get(0)?,
            cases: row.get(1)?,
            url: String::new(),
        })
    })?;

    let entries: Vec<TestEntry> = rows.filter_map(|entry| entry.ok()).collect();
    Ok(entries)
}

fn get_e2e_tests(conn: &Connection, project_id: i64) -> Result<Vec<TestEntry>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT ut.path, COUNT(DISTINCT tc.name) as cases FROM e2e_tests ut
          LEFT JOIN test_cases tc on ut.OID = tc.test_id
        WHERE ut.project_id=:project_id
        GROUP BY ut.path
    "#,
    )?;

    let rows = stmt.query_map(named_params! {":project_id": project_id}, |row| {
        Ok(TestEntry {
            path: row.get(0)?,
            cases: row.get(1)?,
            url: String::new(),
        })
    })?;

    let entries: Vec<TestEntry> = rows.filter_map(|entry| entry.ok()).collect();
    Ok(entries)
}
