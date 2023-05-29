use birdview::{run, Config};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

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
        dir: PathBuf,

        /// Inspect test files
        #[arg(short, long)]
        tests: bool,

        /// Inspect dependencies
        #[arg(short, long)]
        deps: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    match &cli.command {
        Some(Commands::Inspect { dir, tests, deps }) => {
            println!("workspace: {}", dir.display());

            let config = Config {
                working_dir: PathBuf::from(dir),
                inspect_tests: *tests,
                inspect_deps: *deps,
            };

            if let Err(e) = run(config) {
                eprintln!("Application error {e}");
                process::exit(1);
            }
        }
        None => {}
    }
}
