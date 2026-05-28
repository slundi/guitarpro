mod command_info;
mod loader;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "score_tool",
    about = "Inspect and process Guitar Pro score files",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Print metadata, track listing, and timeline for a score file
    Info(command_info::InfoArgs),
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Info(args) => command_info::run(&args),
    };
    if let Err(e) = result {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
