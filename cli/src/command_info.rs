use bpaf::Bpaf;
use guitarpro::audio::midi::CHANNEL_DEFAULT_NAMES;
use guitarpro::{DirectionSign, Song};
use serde::Serialize;

/// Print metadata, track listing, and timeline for a score file
#[derive(Bpaf, Debug)]
#[bpaf(command("info"))]
pub struct InfoArgs {
    /// Input file path (.gp3, .gp4, .gp5, .gp, .gpx)
    #[bpaf(short, long, argument("FILE"))]
    pub input: String,

    /// Print as JSON instead of human-readable text
    #[bpaf(long, switch)]
    pub json: bool,
}

pub fn run(args: &InfoArgs) -> anyhow::Result<()> {
    let (song, fmt) = crate::loader::load_song(&args.input)?;
    if args.json {
        let out = build_json(&song, &args.input, &fmt);
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        print_text(&song, &args.input, &fmt);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn midi_to_note(midi: i8) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let m = midi as u8;
    let octave = (m / 12) as i32 - 1;
    format!("{}{}", NAMES[(m % 12) as usize], octave)
}

fn instrument_name(program: i32) -> &'static str {
    CHANNEL_DEFAULT_NAMES[program.clamp(0, 127) as usize]
}

fn direction_label(d: &DirectionSign) -> &'static str {
    match d {
        DirectionSign::Coda => "Coda",
        DirectionSign::DoubleCoda => "Double Coda",
        DirectionSign::Segno => "Segno",
        DirectionSign::SegnoSegno => "Segno Segno",
        DirectionSign::Fine => "Fine",
        DirectionSign::DaCapo => "D.C.",
        DirectionSign::DaCapoAlCoda => "D.C. al Coda",
        DirectionSign::DaCapoAlDoubleCoda => "D.C. al Double Coda",
        DirectionSign::DaCapoAlFine => "D.C. al Fine",
        DirectionSign::DaSegno => "D.S.",
        DirectionSign::DaSegnoAlCoda => "D.S. al Coda",
        DirectionSign::DaSegnoAlDoubleCoda => "D.S. al Double Coda",
        DirectionSign::DaSegnoAlFine => "D.S. al Fine",
        DirectionSign::DaSegnoSegno => "D.S.S.",
        DirectionSign::DaSegnoSegnoAlCoda => "D.S.S. al Coda",
        DirectionSign::DaSegnoSegnoAlDoubleCoda => "D.S.S. al Double Coda",
        DirectionSign::DaSegnoSegnoAlFine => "D.S.S. al Fine",
        DirectionSign::DaCoda => "Da Coda",
        DirectionSign::DaDoubleCoda => "Da Double Coda",
    }
}

/// Decode an 8-bit volta mask into a list of 1-based ending numbers.
fn decode_volta_mask(bits: u8) -> Vec<u8> {
    (0u8..8)
        .filter(|&i| bits & (1 << i) != 0)
        .map(|i| i + 1)
        .collect()
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        let mut end = max;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        &s[..end]
    }
}

// ---------------------------------------------------------------------------
// Human-readable text output
// ---------------------------------------------------------------------------

pub fn print_text(song: &Song, path: &str, fmt: &str) {
    let ver = song.version.number;

    // ---- Metadata ----------------------------------------------------------
    println!("=== Metadata ===");
    println!("{:<16} {}", "File:", path);
    let fmt_label = match fmt {
        "GPX" => "Guitar Pro 6 (GPX)".to_owned(),
        "GP" => "Guitar Pro 7+ (GP)".to_owned(),
        _ => format!(
            "Guitar Pro {} ({} v{}.{}.{})",
            ver.0, fmt, ver.0, ver.1, ver.2
        ),
    };
    println!("{:<16} {}", "Format:", fmt_label);

    let string_fields: &[(&str, &str)] = &[
        ("Title:", &song.name),
        ("Subtitle:", &song.subtitle),
        ("Artist:", &song.artist),
        ("Album:", &song.album),
        ("Composer:", &song.author),
        ("Words by:", &song.words),
        ("Copyright:", &song.copyright),
        ("Transcribed by:", &song.writer),
        ("Instructions:", &song.instructions),
        ("Comments:", &song.comments),
    ];
    for (label, value) in string_fields {
        if !value.is_empty() {
            println!("{:<16} {}", label, value);
        }
    }
    if !song.notice.is_empty() {
        println!("{:<16} {}", "Notice:", song.notice.join(" | "));
    }
    println!("{:<16} {} BPM", "Tempo:", song.tempo);
    println!("{:<16} {}", "Key:", song.key);
    println!("{:<16} {}", "Measures:", song.measure_headers.len());
    println!("{:<16} {}", "Tracks:", song.tracks.len());

    // ---- Tracks ------------------------------------------------------------
    println!();
    println!("=== Tracks ===");

    const W_NAME: usize = 24;
    const W_KIND: usize = 10;
    const W_TUNING: usize = 22;
    const W_INSTR: usize = 24;

    println!(
        "  {:>2}  {:<name$}  {:<kind$}  {:>8}  {:<tuning$}  {:>6}  {:>3}  {:<instr$}  Flags",
        "#",
        "Name",
        "Type",
        "Measures",
        "Tuning",
        "Voices",
        "Ch",
        "Instrument",
        name = W_NAME,
        kind = W_KIND,
        tuning = W_TUNING,
        instr = W_INSTR,
    );
    println!(
        "{}",
        "-".repeat(
            2 + 2
                + 2
                + W_NAME
                + 2
                + W_KIND
                + 2
                + 8
                + 2
                + W_TUNING
                + 2
                + 6
                + 2
                + 3
                + 2
                + W_INSTR
                + 2
                + 5
        )
    );

    for (i, track) in song.tracks.iter().enumerate() {
        let kind = if track.percussion_track {
            "Percussion"
        } else if track.twelve_stringed_guitar_track {
            "12-string"
        } else if track.banjo_track {
            "Banjo"
        } else {
            "Stringed"
        };

        let channel = song.channels.get(track.channel_index);
        let (ch_num, prog) = channel
            .map(|c| (c.channel, c.instrument))
            .unwrap_or((0, 25));
        let instr = if track.percussion_track {
            "Drums / Percussion"
        } else {
            instrument_name(prog)
        };

        let max_voices = track
            .measures
            .iter()
            .map(|m| m.voices.iter().filter(|v| !v.beats.is_empty()).count())
            .max()
            .unwrap_or(0);

        let tuning = if track.percussion_track {
            "(percussion)".to_owned()
        } else {
            track
                .strings
                .iter()
                .map(|&(_, midi)| midi_to_note(midi))
                .collect::<Vec<_>>()
                .join(" ")
        };

        let mut flags: Vec<&str> = Vec::new();
        if track.solo {
            flags.push("solo");
        }
        if track.mute {
            flags.push("mute");
        }
        if !track.visible {
            flags.push("hidden");
        }

        println!(
            "  {:>2}  {:<name$}  {:<kind$}  {:>8}  {:<tuning$}  {:>6}  {:>3}  {:<instr$}  {}",
            i + 1,
            truncate(&track.name, W_NAME),
            kind,
            track.measures.len(),
            truncate(&tuning, W_TUNING),
            max_voices,
            ch_num + 1,
            truncate(instr, W_INSTR),
            flags.join(","),
            name = W_NAME,
            kind = W_KIND,
            tuning = W_TUNING,
            instr = W_INSTR,
        );
    }

    // ---- Timeline ----------------------------------------------------------
    println!();
    println!("=== Timeline ===");
    println!(
        "  {:>4}  {:>5}  {:>5}  {:<14}  {:<20}  Navigation",
        "Bar", "Time", "Tempo", "Key", "Marker"
    );
    println!("{}", "-".repeat(80));

    let mut prev_time = (0i8, 0u16);
    let mut prev_tempo: i32 = 0;
    let mut prev_key = String::new();

    for mh in &song.measure_headers {
        let ts = &mh.time_signature;
        let is_first = mh.number == 1;
        let time_changed = ts.numerator != prev_time.0 || ts.denominator.value != prev_time.1;

        let effective_tempo = if mh.tempo == 0 && is_first {
            song.tempo as i32
        } else {
            mh.tempo
        };
        let tempo_changed = effective_tempo != 0 && effective_tempo != prev_tempo;

        let key_str = mh.key_signature.to_string();
        let key_changed = key_str != prev_key;

        let has_marker = mh.marker.is_some();
        let has_nav = mh.repeat_open
            || mh.repeat_close >= 0
            || mh.repeat_alternative != 0
            || mh.direction.is_some();

        if is_first || time_changed || tempo_changed || key_changed || has_marker || has_nav {
            let time_col = if is_first || time_changed {
                format!("{}/{}", ts.numerator, ts.denominator.value)
            } else {
                String::new()
            };
            let tempo_col = if is_first || tempo_changed {
                effective_tempo.to_string()
            } else {
                String::new()
            };
            let key_col = if is_first || key_changed {
                key_str.clone()
            } else {
                String::new()
            };
            let marker_col = mh.marker.as_ref().map(|m| m.title.as_str()).unwrap_or("");

            let mut nav: Vec<String> = Vec::new();
            if mh.repeat_open {
                nav.push("|:".to_owned());
            }
            if mh.repeat_close >= 0 {
                nav.push(format!(":|x{}", mh.repeat_close + 1));
            }
            if mh.repeat_alternative != 0 {
                let endings = decode_volta_mask(mh.repeat_alternative);
                let labels: Vec<String> = endings.iter().map(|n| n.to_string()).collect();
                nav.push(format!("[{}.]", labels.join(",")));
            }
            if let Some(dir) = &mh.direction {
                nav.push(direction_label(dir).to_owned());
            }

            println!(
                "  {:>4}  {:>5}  {:>5}  {:<14}  {:<20}  {}",
                mh.number,
                time_col,
                tempo_col,
                key_col,
                truncate(marker_col, 20),
                nav.join("  "),
            );
        }

        if is_first || time_changed {
            prev_time = (ts.numerator, ts.denominator.value);
        }
        if is_first || tempo_changed {
            prev_tempo = effective_tempo;
        }
        if is_first || key_changed {
            prev_key = key_str;
        }
    }
}

// ---------------------------------------------------------------------------
// JSON output
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct InfoOutput {
    file: String,
    format: String,
    version: [u8; 3],
    title: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    subtitle: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    artist: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    album: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    composer: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    words: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    copyright: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    transcriber: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    instructions: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    comments: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    notice: Vec<String>,
    initial_tempo: i16,
    initial_key: String,
    measure_count: usize,
    track_count: usize,
    tracks: Vec<TrackJson>,
    timeline: Vec<MeasureJson>,
}

#[derive(Serialize)]
struct TrackJson {
    index: usize,
    name: String,
    kind: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tuning: Vec<StringJson>,
    frets: u8,
    measure_count: usize,
    max_voices: usize,
    midi_channel: u8,
    midi_program: i32,
    instrument: String,
    solo: bool,
    mute: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    hidden: bool,
}

#[derive(Serialize)]
struct StringJson {
    number: i8,
    midi: i8,
    note: String,
}

#[derive(Serialize)]
struct MeasureJson {
    number: u16,
    time_signature: String,
    tempo: i32,
    key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    marker: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    navigation: Vec<String>,
}

fn build_json(song: &Song, path: &str, fmt: &str) -> InfoOutput {
    let ver = song.version.number;

    let tracks = song
        .tracks
        .iter()
        .enumerate()
        .map(|(i, track)| {
            let kind = if track.percussion_track {
                "percussion"
            } else if track.twelve_stringed_guitar_track {
                "12-string"
            } else if track.banjo_track {
                "banjo"
            } else {
                "stringed"
            };

            let channel = song.channels.get(track.channel_index);
            let (ch_num, prog) = channel
                .map(|c| (c.channel, c.instrument))
                .unwrap_or((0, 25));
            let instr = if track.percussion_track {
                "Drums / Percussion".to_owned()
            } else {
                instrument_name(prog).to_owned()
            };

            let max_voices = track
                .measures
                .iter()
                .map(|m| m.voices.iter().filter(|v| !v.beats.is_empty()).count())
                .max()
                .unwrap_or(0);

            let tuning: Vec<StringJson> = if track.percussion_track {
                Vec::new()
            } else {
                track
                    .strings
                    .iter()
                    .map(|&(num, midi)| StringJson {
                        number: num,
                        midi,
                        note: midi_to_note(midi),
                    })
                    .collect()
            };

            TrackJson {
                index: i + 1,
                name: track.name.clone(),
                kind: kind.to_owned(),
                tuning,
                frets: track.fret_count,
                measure_count: track.measures.len(),
                max_voices,
                midi_channel: ch_num + 1,
                midi_program: prog,
                instrument: instr,
                solo: track.solo,
                mute: track.mute,
                hidden: !track.visible,
            }
        })
        .collect();

    let mut prev_time = (0i8, 0u16);
    let mut prev_tempo: i32 = 0;
    let mut prev_key = String::new();

    let timeline = song
        .measure_headers
        .iter()
        .map(|mh| {
            let ts = &mh.time_signature;
            let is_first = mh.number == 1;
            let time_changed = ts.numerator != prev_time.0 || ts.denominator.value != prev_time.1;
            let effective_tempo = if mh.tempo == 0 && is_first {
                song.tempo as i32
            } else {
                mh.tempo
            };
            let tempo_changed = effective_tempo != 0 && effective_tempo != prev_tempo;
            let key_str = mh.key_signature.to_string();
            let key_changed = key_str != prev_key;

            if is_first || time_changed {
                prev_time = (ts.numerator, ts.denominator.value);
            }
            if is_first || tempo_changed {
                prev_tempo = effective_tempo;
            }
            if is_first || key_changed {
                prev_key = key_str.clone();
            }

            let mut nav: Vec<String> = Vec::new();
            if mh.repeat_open {
                nav.push("repeat_open".to_owned());
            }
            if mh.repeat_close >= 0 {
                nav.push(format!("repeat_close:{}", mh.repeat_close + 1));
            }
            if mh.repeat_alternative != 0 {
                let endings = decode_volta_mask(mh.repeat_alternative);
                let labels: Vec<String> = endings.iter().map(|n| n.to_string()).collect();
                nav.push(format!("volta:{}", labels.join(",")));
            }
            if let Some(dir) = &mh.direction {
                nav.push(direction_label(dir).to_owned());
            }

            MeasureJson {
                number: mh.number,
                time_signature: format!("{}/{}", ts.numerator, ts.denominator.value),
                tempo: prev_tempo,
                key: prev_key.clone(),
                marker: mh.marker.as_ref().map(|m| m.title.clone()),
                navigation: nav,
            }
        })
        .collect();

    InfoOutput {
        file: path.to_owned(),
        format: fmt.to_owned(),
        version: [ver.0, ver.1, ver.2],
        title: song.name.clone(),
        subtitle: song.subtitle.clone(),
        artist: song.artist.clone(),
        album: song.album.clone(),
        composer: song.author.clone(),
        words: song.words.clone(),
        copyright: song.copyright.clone(),
        transcriber: song.writer.clone(),
        instructions: song.instructions.clone(),
        comments: song.comments.clone(),
        notice: song.notice.clone(),
        initial_tempo: song.tempo,
        initial_key: song.key.to_string(),
        measure_count: song.measure_headers.len(),
        track_count: song.tracks.len(),
        tracks,
        timeline,
    }
}
