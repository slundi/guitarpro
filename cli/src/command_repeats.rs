use bpaf::Bpaf;
use guitarpro::analysis::repeats::{PlayedBar, RepeatsReport, track_simile_runs};
use guitarpro::convert::optimized::legacy::build_timeline;
use serde::Serialize;

use crate::command_convert::{ConvertFormats, encode_song, load_as_song, resolve_formats};
use crate::loader::load_song_and_score;

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

/// All analysis results bundled for output.
struct Analysis {
    path: String,
    report: RepeatsReport,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(args: &RepeatsArgs) -> anyhow::Result<()> {
    if args.expand && args.output.is_none() {
        anyhow::bail!("--expand requires --output <path>");
    }

    let (song, _fmt, score) = load_song_and_score(&args.input)?;

    // ---------- 5a: global navigation events ----------
    // Only a GP file's song (one without a score) has the one direction
    // sign its reader kept.
    let version = score.is_none().then_some(song.version.number);
    // Repeats and endings from the score's own timeline where there is
    // one: the song loses other formats' endings past pass 8, their texts
    // and repeat counts.
    let timeline = match score {
        Some(score) => score.score.timeline,
        None => build_timeline(&song),
    };
    // ---------- 5b: simile marks (with the rest of the report) ----------
    let report = RepeatsReport::new(&song, &timeline, version, args.track.as_deref());
    let analysis = Analysis {
        path: args.input.clone(),
        report,
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
// 5b helpers — expand simile marks in place
// ---------------------------------------------------------------------------

fn expand_simile_marks(song: &mut guitarpro::Song) {
    for track in &mut song.tracks {
        // In order: a run's source may be an earlier run, expanded by then.
        for run in track_simile_runs(track) {
            if run.source_len == 0 {
                continue;
            }
            // Replace each bar in the run with a clone of its source bar,
            // the source block repeated in turn.
            for offset in 0..run.run_len {
                let src_idx = run.source_start + offset % run.source_len;
                let target = run.run_start + offset;
                // Clone source voices into the target measure, preserving header fields
                let src_voices = track.measures[src_idx].voices.clone();
                track.measures[target].voices = src_voices;
                track.measures[target].simile_mark = None;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Text output
// ---------------------------------------------------------------------------

fn print_text(a: &Analysis) {
    let Analysis { path, report } = a;
    let RepeatsReport {
        written_measures: written,
        sounding_measures: sounding,
        sounding_includes_jumps: has_jumps,
        sounding_skips_jumps: unfollowed_jumps,
        sounding_truncated: truncated,
        navigation_events: nav_events,
        repeat_blocks: blocks,
        play_sequence: play_seq,
        simile_runs,
        ..
    } = report;

    println!("=== Repeats: {} ===\n", path);
    println!("Written measures : {}", written);
    if *has_jumps {
        println!("Sounding measures: {} (jumps included)", sounding);
    } else {
        println!("Sounding measures: {}", sounding);
    }
    if *unfollowed_jumps {
        println!(
            "  (without jumps that can't be followed: D.S.S., double coda, D.S. without a segno)"
        );
    }
    if *truncated {
        println!("  (cut short: the repeats don't resolve to an end)");
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
            if let Some(written) = ev.repeat_close {
                let plays = ev.repeat_plays.unwrap_or(written);
                parts.push(played_times(":|×", plays, written));
            }
            if !ev.volta.is_empty() {
                let s: Vec<String> = ev.volta.iter().map(|v| v.to_string()).collect();
                parts.push(format!("[{}.]", s.join(",")));
            }
            parts.extend(ev.directions.iter().cloned());
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
                "  Block {}: bars {}–{} ({} bar{}) {}",
                bi + 1,
                bar_open,
                bar_close,
                section_len,
                if section_len == 1 { "" } else { "s" },
                played_times("×", block.plays, block.total_plays),
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
    println!("--- Play Sequence ---");
    if play_seq.is_empty() {
        println!("  (empty)");
    } else {
        // A short score with long nested repeats plays up to a million
        // bars: the analysis keeps only the first ones.
        let seq_str = compact_play_sequence(play_seq);
        for line in &seq_str {
            println!("  {}", line);
        }
        if *sounding > play_seq.len() {
            println!("  … ({} more)", sounding - play_seq.len());
        }
    }
    println!();

    // Simile marks
    println!("--- Per-Track Simile Marks ---");
    if simile_runs.is_empty() {
        println!("  (none)");
    } else {
        for run in simile_runs {
            let (first, last) = run.bars();
            let source = match run.source_bars() {
                Some((from, to)) => format!("repeat bars {from}–{to}"),
                None => "nothing to repeat".to_owned(),
            };
            println!(
                "  Track \"{}\": bars {}–{} → {}  ({})",
                run.track_name, first, last, source, run.kind,
            );
        }
    }
    println!();
}

/// A repeat's passes after `prefix`: as played, and as written when its
/// endings play more.
fn played_times(prefix: &str, plays: u32, written: u32) -> String {
    if plays == written {
        format!("{prefix}{plays}")
    } else {
        format!("{prefix}{plays} (written ×{written})")
    }
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
struct JsonOutput<'a> {
    file: &'a str,
    #[serde(flatten)]
    report: &'a RepeatsReport,
}

fn print_json(a: &Analysis) -> anyhow::Result<()> {
    let out = JsonOutput {
        file: &a.path,
        report: &a.report,
    };
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}
