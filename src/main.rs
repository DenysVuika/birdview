use birdview::config::Config;
use birdview::run;
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

        /// Run all inspections
        #[arg(long)]
        all: bool,

        /// Inspect test files
        #[arg(short, long)]
        tests: bool,

        /// Inspect dependencies
        #[arg(short, long)]
        packages: bool,

        /// Verbose output
        #[arg(long)]
        verbose: bool,

        /// Output report file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    match &cli.command {
        Some(Commands::Inspect {
            dir,
            tests,
            packages,
            all,
            verbose,
            output,
        }) => {
            let config = Config {
                working_dir: PathBuf::from(dir),
                inspect_tests: *all | *tests,
                inspect_packages: *all | *packages,
                verbose: *verbose,
                output: output.to_owned(),
            };

            if let Err(e) = run(&config) {
                eprintln!("Application error {e}");
                process::exit(1);
            }
        }
        None => {}
    }
}
