use bpaf::Bpaf;
use serde::Serialize;

use guitarpro::DirectionSign;

use crate::command_convert::{ConvertFormats, encode_song, load_as_song, resolve_formats};
use crate::loader::load_song;

// ---------------------------------------------------------------------------
// CLI args
// ---------------------------------------------------------------------------

/// Analyse repeat structures and per-track simile marks
#[derive(Bpaf, Debug)]
#[bpaf(command("repeats"))]
pub struct RepeatsArgs {
    /// Input score file
    #[bpaf(short, long, argument("PATH"))]
    pub input: String,

    /// Print results as JSON
    #[bpaf(long, switch)]
    pub json: bool,

    /// Filter simile-mark analysis to a specific track (substring match)
    #[bpaf(long, argument("NAME"))]
    pub track: Option<String>,

    /// Emit a new file with all simile marks replaced by their referenced content
    #[bpaf(long, switch)]
    pub expand: bool,

    /// Output path when using --expand
    #[bpaf(short, long, argument("PATH"))]
    pub output: Option<String>,

    /// Force output format when using --expand: gp3, gp4, gp5, gpx, gp, xml, score
    #[bpaf(long("to"), argument("FORMAT"))]
    pub to: Option<String>,

    /// Force input format: gp3, gp4, gp5, gpx, gp, xml, score
    #[bpaf(long("from"), argument("FORMAT"))]
    pub from: Option<String>,
}

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct NavEvent {
    /// 1-based bar number
    bar: u16,
    repeat_open: bool,
    /// None = no close; Some(n) = play n times total (n ≥ 1)
    repeat_close: Option<u32>,
    /// 1-based volta numbers decoded from bitmask
    volta: Vec<u8>,
    direction: Option<String>,
    marker: Option<String>,
}

#[derive(Debug)]
struct RepeatBlock {
    /// 0-based index of the measure with `repeat_open`
    open_idx: usize,
    /// 0-based index of the measure with `repeat_close`
    close_idx: usize,
    /// Total plays (repeat_close + 1)
    total_plays: u32,
    /// Volta bracket measures: (0-based idx, volta_numbers)
    voltas: Vec<(usize, Vec<u8>)>,
}

#[derive(Debug, Clone)]
struct PlayedBar {
    /// 1-based bar number
    bar_number: u16,
    /// 1-based pass number within the innermost active repeat block
    pass: u32,
}

#[derive(Debug)]
struct SimileRun {
    track_name: String,
    /// 0-based index of first bar in the run (within track.measures)
    run_start: usize,
    /// Length of the run in bars
    run_len: usize,
    /// 0-based index of the source bars being referenced
    source_start: usize,
    /// "Simple", "Double (2-bar)", or custom string
    kind: String,
}

/// All analysis results bundled for output.
struct Analysis {
    path: String,
    written: usize,
    sounding: usize,
    has_jumps: bool,
    nav_events: Vec<NavEvent>,
    blocks: Vec<RepeatBlock>,
    play_seq: Vec<PlayedBar>,
    simile_runs: Vec<SimileRun>,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(args: &RepeatsArgs) -> anyhow::Result<()> {
    if args.expand && args.output.is_none() {
        anyhow::bail!("--expand requires --output <path>");
    }

    let (song, _fmt) = load_song(&args.input)?;

    // ---------- 5a: global navigation events ----------
    let nav_events = collect_nav_events(&song.measure_headers);
    let blocks = find_repeat_blocks(&song.measure_headers);
    let play_seq = expand_simple_repeats(&song.measure_headers);

    let written = song.measure_headers.len();
    let sounding = play_seq.len();
    let has_jumps = nav_events.iter().any(|e| {
        e.direction
            .as_deref()
            .map(|d| d.starts_with("D.") || d.starts_with("Da"))
            .unwrap_or(false)
    });

    // ---------- 5b: simile marks ----------
    let simile_runs = collect_simile_runs(&song, args.track.as_deref());

    let analysis = Analysis {
        path: args.input.clone(),
        written,
        sounding,
        has_jumps,
        nav_events,
        blocks,
        play_seq,
        simile_runs,
    };

    if args.json {
        print_json(&analysis)?;
    } else {
        print_text(&analysis);
    }

    // ---------- --expand: replace simile marks with real content ----------
    if args.expand {
        let output = args.output.as_ref().unwrap();
        let ConvertFormats { src, dst } = resolve_formats(
            args.from.as_deref(),
            &args.input,
            args.to.as_deref(),
            output,
        )?;
        let mut song = load_as_song(&args.input, src)?;
        expand_simile_marks(&mut song);
        let bytes = encode_song(&song, dst)?;
        std::fs::write(output, &bytes)
            .map_err(|e| anyhow::anyhow!("cannot write '{}': {}", output, e))?;
        eprintln!("Written (simile marks expanded): {}", output);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// 5a helpers — navigation events
// ---------------------------------------------------------------------------

fn collect_nav_events(headers: &[guitarpro::MeasureHeader]) -> Vec<NavEvent> {
    headers
        .iter()
        .filter(|mh| {
            mh.repeat_open
                || mh.repeat_close >= 0
                || mh.repeat_alternative != 0
                || mh.direction.is_some()
                || mh.marker.is_some()
        })
        .map(|mh| NavEvent {
            bar: mh.number,
            repeat_open: mh.repeat_open,
            repeat_close: if mh.repeat_close >= 0 {
                Some((mh.repeat_close + 1) as u32)
            } else {
                None
            },
            volta: decode_volta_mask(mh.repeat_alternative),
            direction: mh.direction.as_ref().map(direction_label),
            marker: mh.marker.as_ref().map(|m| m.title.clone()),
        })
        .collect()
}

fn decode_volta_mask(bits: u8) -> Vec<u8> {
    (0u8..8)
        .filter(|&i| bits & (1 << i) != 0)
        .map(|i| i + 1)
        .collect()
}

fn direction_label(d: &DirectionSign) -> String {
    match d {
        DirectionSign::Coda => "Coda".to_owned(),
        DirectionSign::DoubleCoda => "Double Coda".to_owned(),
        DirectionSign::Segno => "Segno".to_owned(),
        DirectionSign::SegnoSegno => "Segno Segno".to_owned(),
        DirectionSign::Fine => "Fine".to_owned(),
        DirectionSign::DaCapo => "Da Capo".to_owned(),
        DirectionSign::DaCapoAlCoda => "D.C. al Coda".to_owned(),
        DirectionSign::DaCapoAlDoubleCoda => "D.C. al Double Coda".to_owned(),
        DirectionSign::DaCapoAlFine => "D.C. al Fine".to_owned(),
        DirectionSign::DaSegno => "D.S.".to_owned(),
        DirectionSign::DaSegnoAlCoda => "D.S. al Coda".to_owned(),
        DirectionSign::DaSegnoAlDoubleCoda => "D.S. al Double Coda".to_owned(),
        DirectionSign::DaSegnoAlFine => "D.S. al Fine".to_owned(),
        DirectionSign::DaSegnoSegno => "D.S.S.".to_owned(),
        DirectionSign::DaSegnoSegnoAlCoda => "D.S.S. al Coda".to_owned(),
        DirectionSign::DaSegnoSegnoAlDoubleCoda => "D.S.S. al Double Coda".to_owned(),
        DirectionSign::DaSegnoSegnoAlFine => "D.S.S. al Fine".to_owned(),
        DirectionSign::DaCoda => "Da Coda".to_owned(),
        DirectionSign::DaDoubleCoda => "Da Double Coda".to_owned(),
    }
}

// ---------------------------------------------------------------------------
// 5a helpers — repeat blocks
// ---------------------------------------------------------------------------

fn find_repeat_blocks(headers: &[guitarpro::MeasureHeader]) -> Vec<RepeatBlock> {
    let mut blocks = Vec::new();
    let mut open_stack: Vec<usize> = Vec::new(); // stack of open_idx

    for (i, mh) in headers.iter().enumerate() {
        if mh.repeat_open {
            open_stack.push(i);
        }
        if mh.repeat_close >= 0 {
            let open_idx = open_stack.pop().unwrap_or(0);
            let total_plays = (mh.repeat_close + 1) as u32;

            // Collect volta measures within this block
            let voltas: Vec<(usize, Vec<u8>)> = headers[open_idx..=i]
                .iter()
                .enumerate()
                .filter(|(_, h)| h.repeat_alternative != 0)
                .map(|(rel, h)| (open_idx + rel, decode_volta_mask(h.repeat_alternative)))
                .collect();

            blocks.push(RepeatBlock {
                open_idx,
                close_idx: i,
                total_plays,
                voltas,
            });
        }
    }
    blocks
}

// ---------------------------------------------------------------------------
// 5a helpers — play sequence expansion (handles |: :|×N + volta)
// ---------------------------------------------------------------------------

fn expand_simple_repeats(headers: &[guitarpro::MeasureHeader]) -> Vec<PlayedBar> {
    // Stack: (repeat_start_idx, current_pass, total_passes)
    let mut stack: Vec<(usize, u32, u32)> = Vec::new();
    let mut result: Vec<PlayedBar> = Vec::new();
    let mut i = 0usize;
    const SAFETY: usize = 10_000;

    while i < headers.len() && result.len() < SAFETY {
        let mh = &headers[i];

        // Push repeat_open only on the first encounter at this index (not when looping back).
        if mh.repeat_open && !stack.iter().any(|(s, _, _)| *s == i) {
            stack.push((i, 1, 1)); // total_passes updated when we see close
        }

        // For implicit repeats (repeat_close without a matching open), start from bar 0.
        if mh.repeat_close >= 0 && stack.is_empty() {
            stack.push((0, 1, 1));
        }

        let pass = stack.last().map(|(_, p, _)| *p).unwrap_or(1);

        // Volta filter: only applies inside a repeat block (non-empty stack).
        // Skips measures whose repeat_alternative bitmask doesn't include the current pass.
        if !stack.is_empty()
            && mh.repeat_alternative != 0
            && (mh.repeat_alternative >> (pass - 1)) & 1 == 0
        {
            // If this skipped bar carries the close barline and we've exhausted passes, pop.
            if mh.repeat_close >= 0 && stack.last().is_some_and(|(_, p, t)| p >= t) {
                stack.pop();
            }
            i += 1;
            continue;
        }

        result.push(PlayedBar {
            bar_number: mh.number,
            pass,
        });

        if mh.repeat_close >= 0 {
            let total = (mh.repeat_close + 1) as u32;
            if let Some(top) = stack.last_mut() {
                top.2 = total; // update total_passes in case this is first encounter
                if top.1 < total {
                    let go_to = top.0;
                    top.1 += 1;
                    i = go_to;
                    continue;
                } else {
                    stack.pop();
                }
            }
        }

        i += 1;
    }

    result
}

// ---------------------------------------------------------------------------
// 5b helpers — simile marks
// ---------------------------------------------------------------------------

fn collect_simile_runs(song: &guitarpro::Song, track_filter: Option<&str>) -> Vec<SimileRun> {
    let mut runs = Vec::new();

    for track in &song.tracks {
        if track_filter.is_some_and(|f| !track.name.to_lowercase().contains(&f.to_lowercase())) {
            continue;
        }

        let measures = &track.measures;
        let mut i = 0usize;

        while i < measures.len() {
            let sm = match &measures[i].simile_mark {
                Some(s) => s.as_str(),
                None => {
                    i += 1;
                    continue;
                }
            };

            // Classify and find run length
            let (step, kind_label) = match sm {
                "Simple" => (1usize, "1-bar (Simple)"),
                "FirstOfDouble" | "SecondOfDouble" => (2usize, "2-bar (Double)"),
                other => {
                    // Unknown — treat as 1-bar run
                    runs.push(SimileRun {
                        track_name: track.name.clone(),
                        run_start: i,
                        run_len: 1,
                        source_start: i.saturating_sub(1),
                        kind: format!("unknown ({})", other),
                    });
                    i += 1;
                    continue;
                }
            };

            // Extend the run while the pattern continues
            // For "Simple": consecutive "Simple" marks
            // For "Double": consecutive FirstOfDouble/SecondOfDouble pairs
            let run_start = i;
            let first_mark_kind = sm;
            while i < measures.len() {
                match measures[i].simile_mark.as_deref() {
                    Some("Simple") if first_mark_kind == "Simple" => i += 1,
                    Some("FirstOfDouble" | "SecondOfDouble")
                        if first_mark_kind == "FirstOfDouble"
                            || first_mark_kind == "SecondOfDouble" =>
                    {
                        i += 1
                    }
                    _ => break,
                }
            }

            let run_len = i - run_start;
            // Source is the equal-length block immediately before the run
            let source_start = run_start.saturating_sub(run_len / step * step);

            runs.push(SimileRun {
                track_name: track.name.clone(),
                run_start,
                run_len,
                source_start,
                kind: kind_label.to_owned(),
            });
        }
    }

    runs
}

// ---------------------------------------------------------------------------
// 5b helpers — expand simile marks in place
// ---------------------------------------------------------------------------

fn expand_simile_marks(song: &mut guitarpro::Song) {
    for track in &mut song.tracks {
        let measures_len = track.measures.len();
        let mut i = 0usize;

        while i < measures_len {
            if track.measures[i].simile_mark.is_none() {
                i += 1;
                continue;
            }

            // Find run
            let run_start = i;
            while i < measures_len && track.measures[i].simile_mark.is_some() {
                i += 1;
            }
            let run_len = i - run_start;
            let source_start = run_start.saturating_sub(run_len);

            // Replace each bar in the run with a clone of the corresponding source bar
            for offset in 0..run_len {
                let src_idx = source_start + (offset % (run_start - source_start).max(1));
                if src_idx < run_start {
                    // Clone source voices into the target measure, preserving header fields
                    let src_voices = track.measures[src_idx].voices.clone();
                    track.measures[run_start + offset].voices = src_voices;
                    track.measures[run_start + offset].simile_mark = None;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Text output
// ---------------------------------------------------------------------------

fn print_text(a: &Analysis) {
    let Analysis {
        path,
        written,
        sounding,
        has_jumps,
        nav_events,
        blocks,
        play_seq,
        simile_runs,
    } = a;

    println!("=== Repeats: {} ===\n", path);
    println!("Written measures : {}", written);
    if *has_jumps {
        println!(
            "Sounding measures: {} (from repeat barlines only; jump signs not included)",
            sounding
        );
    } else {
        println!("Sounding measures: {}", sounding);
    }
    println!();

    // Navigation events
    println!("--- Navigation Events ---");
    if nav_events.is_empty() {
        println!("  (none)");
    } else {
        for ev in nav_events {
            let mut parts: Vec<String> = Vec::new();
            if let Some(m) = &ev.marker {
                parts.push(format!("Marker: \"{}\"", m));
            }
            if ev.repeat_open {
                parts.push("|:".to_owned());
            }
            if let Some(n) = ev.repeat_close {
                parts.push(format!(":|×{}", n));
            }
            if !ev.volta.is_empty() {
                let s: Vec<String> = ev.volta.iter().map(|v| v.to_string()).collect();
                parts.push(format!("[{}.]", s.join(",")));
            }
            if let Some(dir) = &ev.direction {
                parts.push(dir.clone());
            }
            println!("  Bar {:>4}  {}", ev.bar, parts.join("  "));
        }
    }
    println!();

    // Repeat blocks
    println!("--- Repeat Blocks ---");
    if blocks.is_empty() {
        println!("  (none)");
    } else {
        for (bi, block) in blocks.iter().enumerate() {
            let bar_open = block.open_idx + 1;
            let bar_close = block.close_idx + 1;
            let section_len = block.close_idx - block.open_idx + 1;
            println!(
                "  Block {}: bars {}–{} ({} bar{}) ×{}",
                bi + 1,
                bar_open,
                bar_close,
                section_len,
                if section_len == 1 { "" } else { "s" },
                block.total_plays,
            );
            if !block.voltas.is_empty() {
                for (idx, volta_nums) in &block.voltas {
                    let s: Vec<String> = volta_nums.iter().map(|v| v.to_string()).collect();
                    println!("    Volta bar {} → [{}.]", idx + 1, s.join(","));
                }
            }
        }
    }
    println!();

    // Flat play sequence (compact: show runs as ranges)
    println!("--- Play Sequence (simple repeats only) ---");
    if play_seq.is_empty() {
        println!("  (empty)");
    } else {
        let seq_str = compact_play_sequence(play_seq);
        for line in &seq_str {
            println!("  {}", line);
        }
    }
    println!();

    // Simile marks
    println!("--- Per-Track Simile Marks ---");
    if simile_runs.is_empty() {
        println!("  (none)");
    } else {
        for run in simile_runs {
            let run_end = run.run_start + run.run_len;
            let src_end = run.source_start + run.run_len;
            println!(
                "  Track \"{}\": bars {}–{} → repeat bars {}–{}  ({})",
                run.track_name,
                run.run_start + 1,
                run_end,
                run.source_start + 1,
                src_end,
                run.kind,
            );
        }
    }
    println!();
}

/// Compact the play sequence into human-readable ranges.
fn compact_play_sequence(seq: &[PlayedBar]) -> Vec<String> {
    if seq.is_empty() {
        return vec!["(empty)".to_owned()];
    }

    let mut lines: Vec<String> = Vec::new();
    let mut run_start = 0usize;

    while run_start < seq.len() {
        let pass = seq[run_start].pass;
        let mut run_end = run_start;
        // Extend while same pass and consecutive bar numbers
        while run_end + 1 < seq.len()
            && seq[run_end + 1].pass == pass
            && seq[run_end + 1].bar_number == seq[run_end].bar_number + 1
        {
            run_end += 1;
        }

        let first_bar = seq[run_start].bar_number;
        let last_bar = seq[run_end].bar_number;
        let pass_str = if seq.iter().any(|b| b.pass > 1) {
            format!(" (pass {})", pass)
        } else {
            String::new()
        };
        if first_bar == last_bar {
            lines.push(format!("bar {}{}", first_bar, pass_str));
        } else {
            lines.push(format!("bars {}–{}{}", first_bar, last_bar, pass_str));
        }
        run_start = run_end + 1;
    }

    lines
}

// ---------------------------------------------------------------------------
// JSON output
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct JsonOutput {
    file: String,
    written_measures: usize,
    sounding_measures: usize,
    sounding_includes_jumps: bool,
    navigation_events: Vec<JsonNavEvent>,
    repeat_blocks: Vec<JsonBlock>,
    play_sequence: Vec<JsonPlayedBar>,
    simile_runs: Vec<JsonSimileRun>,
}

#[derive(Serialize)]
struct JsonNavEvent {
    bar: u16,
    repeat_open: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    repeat_close: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    volta: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    marker: Option<String>,
}

#[derive(Serialize)]
struct JsonBlock {
    open_bar: usize,
    close_bar: usize,
    total_plays: u32,
    volta_bars: Vec<JsonVoltaBar>,
}

#[derive(Serialize)]
struct JsonVoltaBar {
    bar: usize,
    endings: Vec<u8>,
}

#[derive(Serialize)]
struct JsonPlayedBar {
    bar: u16,
    pass: u32,
}

#[derive(Serialize)]
struct JsonSimileRun {
    track: String,
    bars: String,
    source_bars: String,
    kind: String,
}

fn print_json(a: &Analysis) -> anyhow::Result<()> {
    let Analysis {
        path,
        written,
        sounding,
        has_jumps,
        nav_events,
        blocks,
        play_seq,
        simile_runs,
    } = a;
    let out = JsonOutput {
        file: path.clone(),
        written_measures: *written,
        sounding_measures: *sounding,
        sounding_includes_jumps: *has_jumps,
        navigation_events: nav_events
            .iter()
            .map(|ev| JsonNavEvent {
                bar: ev.bar,
                repeat_open: ev.repeat_open,
                repeat_close: ev.repeat_close,
                volta: ev.volta.clone(),
                direction: ev.direction.clone(),
                marker: ev.marker.clone(),
            })
            .collect(),
        repeat_blocks: blocks
            .iter()
            .map(|b| JsonBlock {
                open_bar: b.open_idx + 1,
                close_bar: b.close_idx + 1,
                total_plays: b.total_plays,
                volta_bars: b
                    .voltas
                    .iter()
                    .map(|(idx, v)| JsonVoltaBar {
                        bar: idx + 1,
                        endings: v.clone(),
                    })
                    .collect(),
            })
            .collect(),
        play_sequence: play_seq
            .iter()
            .map(|pb| JsonPlayedBar {
                bar: pb.bar_number,
                pass: pb.pass,
            })
            .collect(),
        simile_runs: simile_runs
            .iter()
            .map(|r| {
                let run_end = r.run_start + r.run_len;
                let src_end = r.source_start + r.run_len;
                JsonSimileRun {
                    track: r.track_name.clone(),
                    bars: format!("{}-{}", r.run_start + 1, run_end),
                    source_bars: format!("{}-{}", r.source_start + 1, src_end),
                    kind: r.kind.clone(),
                }
            })
            .collect(),
    };
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}
