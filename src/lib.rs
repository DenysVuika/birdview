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
use std::process;

pub async fn run(config: &Config) -> Result<()> {
    let package_json_path = &config.working_dir.join("package.json");
    if !package_json_path.exists() {
        panic!("Cannot find package.json file");
    }

    let project = GitProject::open(&config.working_dir)?;
    log::info!("Current branch: {}", project.branch()?);

    let conn = db::create_connection(&config.output_dir)?;
    let package = PackageJsonFile::from_file(package_json_path)?;

    let name = package.name.unwrap();
    let version = package.version.unwrap();

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
            // TODO: version should be part of the snapshot as package.json values differ in branches/tags
            db::create_project(&conn, &name, &version, &remote_url)?
        }
    };

    // todo: load tags
    let tags = vec![project.branch()?];

    let inspectors: Vec<Box<dyn FileInspector>> = vec![
        Box::new(PackageJsonInspector {}),
        Box::new(TestInspector {}),
        Box::new(AngularInspector {}),
    ];

    log::info!("Processing tags: {:?}", tags);

    // TODO: move to a separate func
    for tag in tags {
        project.checkout(&tag)?;
        let sha = project.sha()?;

        log::info!("Creating tag {}", tag);
        let tag_id = db::create_tag(&conn, pid, &tag)?;

        log::info!("Creating new snapshot for tag `{}`({})", tag, sha);
        let sid = db::create_snapshot(&conn, pid, tag_id, &project)?;

        log::info!("Recording authors");
        let authors = &project.authors()?;
        db::create_authors(&conn, sid, authors)?;

        if let Some(dependencies) = &package.dependencies {
            if let Some(version) = dependencies.get("@angular/core") {
                db::create_ng_version(&conn, sid, version)?;
            }
        }

        run_inspectors(config, &conn, sid, &inspectors, config.verbose, &project)?;
    }

    log::info!("Inspection complete");

    if config.open {
        run_server(config.output_dir.to_owned(), true)
            .await
            .unwrap_or_else(|err| log::error!("{:?}", err));
    }

    Ok(())
}

fn run_inspectors(
    config: &Config,
    connection: &Connection,
    sid: i64,
    inspectors: &[Box<dyn FileInspector>],
    verbose: bool,
    project: &GitProject,
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
                    let remote = &project.remote_url()?;
                    let target = &project.sha()?;
                    let url = format!("{remote}/blob/{target}/{rel_path}");

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
            log::info!(
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
