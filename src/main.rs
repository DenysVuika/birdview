use birdview::config::{Config, OutputFormat};
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
        working_dir: PathBuf,

        /// Run all inspections
        #[arg(long)]
        all: bool,

        /// Inspect test files
        #[arg(short, long)]
        tests: bool,

        /// Inspect dependencies
        #[arg(short, long)]
        packages: bool,

        /// Inspect angular elements
        #[arg(short, long)]
        angular: bool,

        /// Inspect file types
        #[arg(long)]
        types: bool,

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

        /// The output format for the report
        #[arg(value_enum, long, default_value_t=OutputFormat::Html)]
        format: OutputFormat,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    match &cli.command {
        Some(Commands::Inspect {
            working_dir,
            tests,
            packages,
            angular,
            types,
            all,
            verbose,
            output_dir,
            open,
            format,
        }) => {
            let config = Config {
                working_dir: working_dir.to_owned(),
                output_dir: match output_dir {
                    Some(path) => path.to_owned(),
                    None => working_dir.to_owned(),
                },
                inspect_tests: *all | *tests,
                inspect_packages: *all | *packages,
                inspect_angular: *all | *angular,
                inspect_types: *all | *types,
                verbose: *verbose,
                open: *open,
                format: *format,
            };

            if let Err(e) = run(&config) {
                eprintln!("Application error {e}");
                process::exit(1);
            }
        }
        None => {}
    }
}
