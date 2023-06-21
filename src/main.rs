use anyhow::Result;
use birdview::config::Config;
use birdview::run;
use birdview::server::run_server;
use clap::{Parser, Subcommand};
use git2::Repository;
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
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let cli = Cli::parse();

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    match &cli.command {
        Some(Commands::Inspect {
            working_dir,
            verbose,
            output_dir,
            open,
        }) => {
            if working_dir.starts_with("https://") {
                let repo_dir = tempdir().expect("Failed creating temporary dir");

                let url = match working_dir.strip_suffix(".git") {
                    Some(value) => value,
                    None => working_dir,
                };

                println!("Cloning {} => {}", url, repo_dir.path().display());
                let repo = match Repository::clone(url, &repo_dir) {
                    Ok(repo) => repo,
                    Err(e) => {
                        repo_dir.close()?;
                        panic!("failed to clone: {}", e)
                    }
                };
                println!("Branch: {}", repo.head()?.shorthand().unwrap());

                let config = Config {
                    working_dir: repo_dir.path().to_owned(),
                    output_dir: match output_dir {
                        Some(value) => value.to_owned(),
                        None => std::env::current_dir()?,
                    },
                    verbose: *verbose,
                    open: *open,
                };

                if let Err(e) = run(&config) {
                    eprintln!("Application error {e}");
                    repo_dir.close()?;
                    process::exit(1);
                } else {
                    repo_dir.close()?;
                }
            } else {
                let config = Config {
                    working_dir: PathBuf::from(working_dir),
                    output_dir: match output_dir {
                        Some(dir) => dir.to_owned(),
                        None => PathBuf::from(working_dir),
                    },
                    verbose: *verbose,
                    open: *open,
                };

                if let Err(e) = run(&config) {
                    eprintln!("Application error {e}");
                    process::exit(1);
                }
            }
        }
        Some(Commands::Serve { working_dir, open }) => {
            run_server(PathBuf::from(working_dir), *open)
                .await
                .unwrap_or_else(|err| println!("{:?}", err));
        }
        None => {}
    }

    Ok(())
}
