use clap::Parser;
use scorelib::Song;
use scorelib::Track;
use std::fs;
use std::io::Read;
use std::path::Path;

const GUITAR_FILE_MAX_SIZE: usize = 16777216; // 16 MB

#[derive(Parser, Debug)]
#[clap(author="slundi", version, about="Guitar Pro File Parser CLI", long_about = None)]
struct Args {
    /// Input file path (.gp3, .gp4, .gp5)
    #[clap(short, long)]
    input: String,

    /// Show full tablature for the first track
    #[clap(short, long)]
    tab: bool,
}

fn main() {
    let args = Args::parse();
    let path = Path::new(&args.input);

    if !path.exists() {
        eprintln!("Error: File '{}' not found.", args.input);
        std::process::exit(1);
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_uppercase())
        .unwrap_or_else(|| "UNKNOWN".to_string());

    let size = fs::metadata(&args.input)
        .map(|m| m.len() as usize)
        .unwrap_or(0);

    if size > GUITAR_FILE_MAX_SIZE {
        eprintln!("Error: File is too large (> 16MB)");
        std::process::exit(1);
    }

    let mut file = fs::File::open(&args.input).expect("Cannot open file");
    let mut data = Vec::with_capacity(size);
    file.read_to_end(&mut data).expect("Cannot read file");

    let mut song = Song::default();
    let result = match ext.as_str() {
        "GP3" => song.read_gp3(&data),
        "GP4" => song.read_gp4(&data),
        "GP5" => song.read_gp5(&data),
        "GP" => song.read_gp(&data),
        "GPX" => song.read_gpx(&data),
        _ => {
            eprintln!(
                "Error: Unsupported format '{}'. Supported: GP3, GP4, GP5, GP.",
                ext
            );
            std::process::exit(1);
        }
    };
    if let Err(e) = result {
        eprintln!("Error reading file: {}", e);
        std::process::exit(1);
    }

    print_metadata(&song);

    if args.tab {
        if let Some(track) = song.tracks.first() {
            println!("\nGenerating Tablature for Track 1: {}", track.name);
            print_ascii_tab(track);
        } else {
            println!("\nNo tracks found in the song.");
        }
    } else {
        println!("\nTip: Use --tab or -t to see the ASCII tablature.");
    }
}

fn print_metadata(song: &Song) {
    println!("=== Metadata ===");
    println!("Title:       {}", song.name);
    println!("Artist:      {}", song.artist);
    println!("Album:       {}", song.album);
    println!("Author:      {}", song.author);
    println!("Date:        {}", song.date);
    println!("Copyright:   {}", song.copyright);
    println!("Transcriber: {}", song.transcriber);
    println!("Comments:    {}", song.comments);
    println!(
        "Version:     {}.{}.{}",
        song.version.number.0, song.version.number.1, song.version.number.2
    );
    println!("Tracks:      {}", song.tracks.len());
    println!("Tempos:      MixTable items (approx)");
}

fn print_ascii_tab(track: &Track) {
    let num_strings = track.strings.len();
    if num_strings == 0 {
        return;
    }

    // Buffer for each string (reversed because string 1 is highest pitch = top line)
    // Actually track.strings[0] is usually String 1 (High E).
    // Tab lines: 0=High E, 1=B, 2=G ...
    let mut lines: Vec<String> = vec![String::new(); num_strings];

    // Tuning info
    let tuning_names = ["E", "B", "G", "D", "A", "E", "B", "F#"]; // Simple approximation
    for i in 0..num_strings {
        let sc = if i < tuning_names.len() {
            tuning_names[i]
        } else {
            "?"
        };
        lines[i].push_str(&format!("{} |", sc));
    }

    // Iterate measures
    for measure in track.measures.iter() {
        // Start of measure bar
        for line in &mut lines {
            line.push('|');
        }

        if measure.voices.is_empty() {
            // Empty measure pad
            for line in &mut lines {
                line.push_str("----");
            }
            continue;
        }

        // We only verify Voice 0
        let voice = &measure.voices[0];

        for beat in &voice.beats {
            // Determine columns needed for this beat (e.g., 3 chars: "12-" or "-")
            // Check notes in this beat
            let mut col_vals: Vec<String> = vec!["-".to_string(); num_strings];

            for note in &beat.notes {
                // Note string index (1-based usually)
                // If note.string is 1, it corresponds to track.strings[0] (High E) -> lines[0]
                let s_idx = (note.string - 1) as usize;
                if s_idx < num_strings {
                    col_vals[s_idx] = note.value.to_string();
                }
            }

            // Find max width for this column (beat) to align vertical start
            let max_width = col_vals.iter().map(|s| s.len()).max().unwrap_or(1);
            let cell_width = max_width + 1; // +1 for spacing

            for i in 0..num_strings {
                let s = &col_vals[i];
                lines[i].push_str(s);
                // Padding
                for _ in 0..(cell_width - s.len()) {
                    lines[i].push('-');
                }
            }
        }
    }

    // Print lines
    println!();
    for line in lines {
        println!("{}", line);
    }
    println!();
}
