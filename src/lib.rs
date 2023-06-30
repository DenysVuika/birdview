pub mod config;
pub mod db;
pub mod git;
pub mod inspectors;
pub mod logger;
pub mod models;
pub mod server;

use crate::config::Config;
use crate::git::GitProject;
use crate::inspectors::*;
use crate::models::PackageJsonFile;
use crate::server::run_server;
use anyhow::Result;
use ignore::WalkBuilder;
use rusqlite::Connection;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process;

pub async fn run(config: &Config) -> Result<()> {
    let package_json_path = &config.working_dir.join("package.json");
    if !package_json_path.exists() {
        panic!("Cannot find package.json file");
    }

    let project = GitProject::open(&config.working_dir)?;
    if !project.is_clean() {
        panic!("Repository is not clean");
    }

    log::info!("Current branch: {}", project.branch()?);

    let conn = db::create_connection(&config.output_dir)?;
    let name = project_name(&config.working_dir)?;

    let pid = match db::get_project_by_name(&conn, &name) {
        Ok(project_info) => {
            log::info!("Found the project `{}`", &name);
            log::info!("Verifying snapshots...");
            let sha = project.sha()?;

            if let Some(snapshot) = db::get_snapshot_by_sha(&conn, &sha) {
                log::info!(
                    "Snapshot {} for `{}` ({}) is already created.",
                    snapshot.oid,
                    &name,
                    &sha
                );

                if config.open {
                    run_server(config.output_dir.to_owned(), true)
                        .await
                        .unwrap_or_else(|err| log::error!("{err}"));
                }

                process::exit(0);
            }

            project_info.id
        }
        Err(_) => {
            log::info!("Creating project `{}`", &name);
            let remote_url = project.remote_url()?;
            db::create_project(&conn, &name, &remote_url)?
        }
    };

    // Load all tags or just the current branch
    // let mut tags = match config.tags {
    //     true => project.tags(),
    //     false => vec![],
    // };
    // tags.push(project.branch()?);

    let tags: Vec<String> = vec![
        "3.1.0".to_owned(),
        "4.0.0".to_owned(),
        "4.0.0-A.1".to_owned(),
        "4.0.0-A.2".to_owned(),
        "4.0.0-A.3".to_owned(),
        "develop".to_owned(),
    ];

    log::info!("Processing tags: {:?}", tags);
    for tag in tags {
        inspect_tag(&project, pid, &conn, &tag, config.verbose)?;
    }

    log::info!("Inspection complete");

    if config.open {
        log::info!("Running web server");
        run_server(config.output_dir.to_owned(), true)
            .await
            .unwrap_or_else(|err| log::error!("{:?}", err));
    }

    Ok(())
}

fn inspect_tag(
    project: &GitProject,
    pid: i64,
    conn: &Connection,
    tag: &String,
    verbose: bool,
) -> Result<()> {
    project.checkout(tag)?;
    let sha = project.sha()?;

    log::info!("Creating tag {}", tag);
    let tag_id = db::create_tag(conn, pid, tag)?;

    log::info!("Creating new snapshot for tag `{}`({})", tag, sha);
    let sid = db::create_snapshot(conn, pid, tag_id, project)?;

    log::info!("Recording authors");
    let authors = &project.authors()?;
    db::create_authors(conn, sid, authors)?;

    let package_json_path = &project.working_dir.join("package.json");
    let version = angular_version(package_json_path)?;
    log::info!("Detected Angular version: {}", version,);
    db::create_ng_version(conn, sid, &version)?;

    run_inspectors(&project.working_dir, conn, sid, verbose, project)?;

    log::info!("Generating metadata");
    db::generate_metadata(conn, pid, sid)?;

    Ok(())
}

fn run_inspectors(
    working_dir: &PathBuf,
    conn: &Connection,
    sid: i64,
    verbose: bool,
    project: &GitProject,
) -> Result<()> {
    let inspectors: Vec<Box<dyn FileInspector>> = vec![
        Box::new(PackageJsonInspector {}),
        Box::new(TestInspector {}),
        Box::new(AngularInspector {}),
    ];

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
                    let remote = &project.remote_url()?;
                    let target = &project.sha()?;
                    let url = format!("{remote}/blob/{target}/{rel_path}");

                    let options = FileInspectorOptions {
                        sid,
                        path: entry_path.to_path_buf(),
                        relative_path: rel_path.to_owned(),
                        url,
                    };

                    inspector.inspect_file(conn, &options)?;
                    processed = true;
                }
            }
        }

        if verbose {
            log::info!(
                "├── {} {}",
                if processed { '✅' } else { '🔎' },
                entry_path.strip_prefix(working_dir).unwrap().display()
            );
        }
    }

    if !types.is_empty() {
        db::create_file_types(conn, sid, &types)?;
    }

    Ok(())
}

pub fn project_name(working_dir: &Path) -> Result<String> {
    let path = &working_dir.join("package.json");
    if !path.exists() {
        panic!("Cannot find package.json file");
    }

    let package = PackageJsonFile::from_file(path)?;
    let name = package.name.unwrap();

    Ok(name)
}

pub fn angular_version(path: &Path) -> Result<String> {
    let package = PackageJsonFile::from_file(path)?;

    if let Some(dependencies) = &package.dependencies {
        if let Some(version) = dependencies.get("@angular/core") {
            return Ok(version.to_owned());
        }
    }

    Ok("unknown".to_owned())
}
