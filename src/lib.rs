pub mod config;
pub mod db;
pub mod git;
pub mod inspectors;
pub mod models;
pub mod report;

use crate::config::Config;
use crate::git::{get_repository_info, RepositoryInfo};
use crate::inspectors::*;
use crate::models::PackageJsonFile;
use anyhow::Result;
use ignore::WalkBuilder;
use rusqlite::Connection;
use std::collections::HashMap;
use std::ffi::OsStr;

pub fn run(config: &Config) -> Result<()> {
    let package_json_path = &config.working_dir.join("package.json");
    if !package_json_path.exists() {
        panic!("Cannot find package.json file");
    }

    let repo = get_repository_info(&config.working_dir);
    let conn = db::create_connection(&config.output_dir)?;
    let package = PackageJsonFile::from_file(package_json_path)?;

    let name = package.name.unwrap();
    let version = package.version.unwrap();

    let pid = match db::get_project_by_name(&conn, &name) {
        Ok(project) => {
            println!("Found the project. Creating new snapshot.");
            project.id
        }
        Err(_) => {
            println!("No project found. Creating a new one.");
            let origin = repo.as_ref().map(|value| &value.remote_url);
            db::create_project(&conn, &name, &version, origin)?
        }
    };

    let sid = db::create_snapshot(&conn, pid, &repo)?;

    if repo.is_some() {
        let authors = repo.as_ref().map(|value| &value.authors);
        db::create_authors(&conn, sid, authors.unwrap())?;
    }

    if let Some(dependencies) = package.dependencies {
        if let Some(version) = dependencies.get("@angular/core") {
            db::create_ng_version(&conn, sid, version)?;
        }
    }

    let inspectors: Vec<Box<dyn FileInspector>> = vec![
        Box::new(PackageJsonInspector {}),
        Box::new(TestInspector {}),
        Box::new(AngularInspector {}),
    ];

    run_inspectors(config, &conn, sid, inspectors, config.verbose, &repo)?;

    let data = report::generate_report(&conn, sid)?;
    report::save_report(config, data)?;

    println!("Inspection complete");
    Ok(())
}

fn run_inspectors(
    config: &Config,
    connection: &Connection,
    sid: i64,
    inspectors: Vec<Box<dyn FileInspector>>,
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
                        sid,
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
        db::create_file_types(connection, sid, &types)?;
    }

    Ok(())
}
