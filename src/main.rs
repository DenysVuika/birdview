use anyhow::Result;
use birdview::config::Config;
use birdview::server::run_server;
use birdview::{logger, run};
use clap::{Parser, Subcommand};
use git2::Repository;
use git_url_parse::GitUrl;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process;
use tempfile::tempdir;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(arg_required_else_help(true))]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect the workspace
    Inspect {
        /// Workspace directory
        working_dir: String,

        /// Verbose output
        #[arg(long)]
        verbose: bool,

        /// Output dir to store reports.
        /// By default takes the workspace directory value.
        #[arg(short, long)]
        output_dir: Option<PathBuf>,

        /// Inspect all repository tags
        #[arg(long)]
        tags: bool,

        /// Open report in browser where applicable.
        /// Supported for the output formats: html
        #[arg(long)]
        open: bool,
    },
    /// Run internal web server
    Serve {
        /// Workspace directory
        working_dir: String,
        /// Open report in browser where applicable.
        /// Supported for the output formats: html
        #[arg(long)]
        open: bool,
    },
}

#[actix_web::main]
async fn main() -> Result<()> {
    // env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    logger::init_logger();

    let cli = Cli::parse();

    if let Some(config_path) = cli.config.as_deref() {
        log::info!("Value for config: {}", config_path.display());
    }

    match &cli.command {
        Some(Commands::Inspect {
            working_dir,
            verbose,
            output_dir,
            open,
            tags,
        }) => {
            if working_dir.starts_with("https://") {
                let repo_url = GitUrl::parse(working_dir).unwrap();
                let repo_dir = tempdir().expect("Failed creating temporary dir");

                let url = match working_dir.strip_suffix(".git") {
                    Some(value) => value,
                    None => working_dir,
                };

                log::info!("Cloning {} => {}", url, repo_dir.path().display());
                let repo = match Repository::clone(url, &repo_dir) {
                    Ok(repo) => repo,
                    Err(e) => {
                        repo_dir.close()?;
                        panic!("failed to clone: {}", e)
                    }
                };
                log::info!("Branch: {}", repo.head()?.shorthand().unwrap());

                let config = Config {
                    project_name: repo_url.name,
                    working_dir: repo_dir.path().to_owned(),
                    output_dir: match output_dir {
                        Some(value) => value.to_owned(),
                        None => std::env::current_dir()?,
                    },
                    verbose: *verbose,
                    open: *open,
                    tags: *tags,
                };

                run(&config).await.unwrap_or_else(|err| {
                    log::error!("Application error {err}");
                    repo_dir.close().unwrap();
                    process::exit(1);
                });

                // repo_dir.close()?;
            } else {
                let project_name: String = PathBuf::from(working_dir)
                    .file_name()
                    .and_then(OsStr::to_str)
                    .unwrap()
                    .into();
                let config = Config {
                    project_name,
                    working_dir: PathBuf::from(working_dir),
                    output_dir: match output_dir {
                        Some(dir) => dir.to_owned(),
                        None => PathBuf::from(working_dir),
                    },
                    verbose: *verbose,
                    open: *open,
                    tags: *tags,
                };

                println!("{:?}", PathBuf::from(working_dir).file_name());

                run(&config).await.unwrap_or_else(|err| {
                    log::error!("Application error {err}");
                    process::exit(1);
                });
            }
        }
        Some(Commands::Serve { working_dir, open }) => {
            run_server(PathBuf::from(working_dir), *open)
                .await
                .unwrap_or_else(|err| log::error!("{:?}", err));
        }
        None => {}
    }

    Ok(())
}
