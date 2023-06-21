use crate::config::{Config, OutputFormat};
use crate::db;
use crate::db::NgKind;
use anyhow::Result;
use rusqlite::Connection;
use serde_json::{json, Map, Value};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn generate_report(conn: &Connection, sid: i64) -> Result<Map<String, Value>> {
    let mut output = Map::new();
    let snapshot = db::get_snapshot_by_id(conn, sid)?;
    let project = db::get_project_by_snapshot(conn, sid)?;

    output.insert(
        "project".to_owned(),
        json!({
            "name": project.name,
            "version": project.version,
            "created_on": snapshot.created_on,
            "origin": project.origin,
            "branch": snapshot.branch,
            "sha": snapshot.sha
        }),
    );

    let ng_version = db::get_ng_version(conn, sid)?;
    output.insert("angular_version".to_owned(), json!(ng_version));

    match db::get_dependencies(conn, sid) {
        Ok(dependencies) => {
            output.insert("dependencies".to_owned(), json!(dependencies));
        }
        Err(err) => println!("{}", err),
    }

    match db::get_packages(conn, sid) {
        Ok(packages) => {
            output.insert("packages".to_owned(), json!(packages));
        }
        Err(err) => println!("{}", err),
    }

    match get_angular_report(conn, sid) {
        Ok(angular) => {
            output.entry("angular").or_insert(angular);
        }
        Err(err) => println!("{}", err),
    };

    match db::get_file_types(conn, sid) {
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

fn get_angular_report(conn: &Connection, sid: i64) -> Result<Value> {
    let modules = db::get_ng_entities(conn, sid, NgKind::Module)?;
    let components = db::get_ng_entities(conn, sid, NgKind::Component)?;
    let directives = db::get_ng_entities(conn, sid, NgKind::Directive)?;
    let services = db::get_ng_entities(conn, sid, NgKind::Service)?;
    let pipes = db::get_ng_entities(conn, sid, NgKind::Pipe)?;
    let dialogs = db::get_ng_entities(conn, sid, NgKind::Dialog)?;

    Ok(json!({
        "modules": modules,
        "components": components,
        "directives": directives,
        "services": services,
        "pipes": pipes,
        "dialogs": dialogs
    }))
}
