mod command_convert;
mod command_duplicates;
mod command_extract;
mod command_form;
mod command_info;
mod command_repeats;
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
    /// Convert a score file between formats (GP3/4/5/GPX/GP, MusicXML, Optimized)
    Convert(command_convert::ConvertArgs),
    /// Extract one or more tracks into a new score file
    Extract(command_extract::ExtractArgs),
    /// Find duplicate or near-duplicate score files in a directory
    Duplicates(command_duplicates::DuplicatesArgs),
    /// Analyse repeat structures and per-track simile marks
    Repeats(command_repeats::RepeatsArgs),
    /// Detect musical form (verse/chorus/bridge/…) by section similarity
    Form(command_form::FormArgs),
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Info(args) => command_info::run(&args),
        Commands::Convert(args) => command_convert::run(&args),
        Commands::Extract(args) => command_extract::run(&args),
        Commands::Duplicates(args) => command_duplicates::run(&args),
        Commands::Repeats(args) => command_repeats::run(&args),
        Commands::Form(args) => command_form::run(&args),
    };
    if let Err(e) = result {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
