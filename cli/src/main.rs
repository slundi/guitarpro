mod command_convert;
mod command_duplicates;
mod command_extract;
mod command_fingering;
mod command_form;
mod command_info;
mod command_mscz;
mod command_repeats;
mod loader;

use bpaf::{Parser, construct};

#[derive(Debug)]
enum Commands {
    Info(command_info::InfoArgs),
    Convert(command_convert::ConvertArgs),
    Extract(command_extract::ExtractArgs),
    Duplicates(command_duplicates::DuplicatesArgs),
    Repeats(command_repeats::RepeatsArgs),
    Form(command_form::FormArgs),
    Fingering(command_fingering::FingeringArgs),
    Mscz(command_mscz::MsczArgs),
}

fn parse_command() -> Commands {
    let info = command_info::info_args().map(Commands::Info);
    let convert = command_convert::convert_args().map(Commands::Convert);
    let extract = command_extract::extract_args().map(Commands::Extract);
    let duplicates = command_duplicates::duplicates_args().map(Commands::Duplicates);
    let repeats = command_repeats::repeats_args().map(Commands::Repeats);
    let form = command_form::form_args().map(Commands::Form);
    let fingering = command_fingering::fingering_args().map(Commands::Fingering);
    let mscz = command_mscz::mscz_args().map(Commands::Mscz);

    construct!([
        info, convert, extract, duplicates, repeats, form, fingering, mscz
    ])
    .to_options()
    .descr("Inspect and process Guitar Pro score files")
    .version(env!("CARGO_PKG_VERSION"))
    .run()
}

fn main() {
    let result = match parse_command() {
        Commands::Info(args) => command_info::run(&args),
        Commands::Convert(args) => command_convert::run(&args),
        Commands::Extract(args) => command_extract::run(&args),
        Commands::Duplicates(args) => command_duplicates::run(&args),
        Commands::Repeats(args) => command_repeats::run(&args),
        Commands::Form(args) => command_form::run(&args),
        Commands::Fingering(args) => command_fingering::run(&args),
        Commands::Mscz(args) => command_mscz::run(&args),
    };
    if let Err(e) = result {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
