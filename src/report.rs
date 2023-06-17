use crate::config::{Config, OutputFormat};
use crate::db;
use crate::git::RepositoryInfo;
use anyhow::Result;
use rusqlite::Connection;
use serde_json::{json, Map, Value};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn generate_report(
    conn: &Connection,
    project_id: i64,
    repo: &Option<RepositoryInfo>,
) -> Result<Map<String, Value>> {
    let mut output = Map::new();
    let project = db::get_project_by_id(conn, project_id)?;

    let ng_version = db::get_ng_version(conn, project_id)?;
    output.insert("angular_version".to_owned(), json!(ng_version));

    if repo.is_some() {
        output.insert("git".to_owned(), json!(repo));
    }

    output.insert(
        "project".to_owned(),
        json!({
            "name": project.name,
            "version": project.version,
            "created_on": project.created_on,
            "origin": project.origin
        }),
    );

    let warnings = db::get_warnings(conn, project_id).unwrap_or(vec![]);
    output.insert("warnings".to_owned(), json!(warnings));

    match db::get_dependencies(conn, project_id) {
        Ok(dependencies) => {
            output.insert("dependencies".to_owned(), json!(dependencies));
        }
        Err(err) => println!("{}", err),
    }

    match db::get_packages(conn, project_id) {
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

    match db::get_unit_tests(conn, project_id) {
        Ok(tests) => {
            output.entry("unit_tests").or_insert(json!(tests));
        }
        Err(err) => println!("{}", err),
    }

    match db::get_e2e_tests(conn, project_id) {
        Ok(tests) => {
            output.entry("e2e_tests").or_insert(json!(tests));
        }
        Err(err) => println!("{}", err),
    }

    match db::get_file_types(conn, project_id) {
        Ok(types) => {
            output.entry("types").or_insert(json!(types));
        }
        Err(err) => println!("{}", err),
    }

    Ok(output)
}

pub fn save_report(config: &Config, data: Map<String, Value>) -> Result<()> {
    let output_file_path = get_output_file(&config.output_dir, config.format).unwrap();
    let mut output_file = File::create(&output_file_path)?;
    let json_string = serde_json::to_string_pretty(&data)?;

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

fn get_angular_report(conn: &Connection, project_id: i64) -> Result<Value> {
    let modules = db::get_ng_modules(conn, project_id)?;
    let components = db::get_ng_components(conn, project_id)?;
    let directives = db::get_ng_directives(conn, project_id)?;
    let services = db::get_ng_services(conn, project_id)?;
    let pipes = db::get_ng_pipes(conn, project_id)?;
    let dialogs = db::get_ng_dialogs(conn, project_id)?;

    Ok(json!({
        "modules": modules,
        "components": components,
        "directives": directives,
        "services": services,
        "pipes": pipes,
        "dialogs": dialogs
    }))
}
