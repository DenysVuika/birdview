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
    /// does testing things
    Test {
        workspace: PathBuf,
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    match &cli.command {
        Some(Commands::Test { workspace, list }) => {
            println!("workspace: {}", workspace.display());

            if *list {
                println!("Printing testing lists...");
                if let Err(e) = run(workspace) {
                    eprintln!("Application error {e}");
                    process::exit(1);
                }
            } else {
                println!("Not printing testing lists...");
            }
        }
        None => {}
    }
}
