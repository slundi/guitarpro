use std::collections::HashMap;

use anyhow::Context;
use bpaf::Bpaf;
use serde::Serialize;

use guitarpro::{
    analysis::fingering::{FingerAssignment, FingerRole, suggest_fingering},
    convert::optimized::legacy::legacy_song_to_loaded_score,
    model::optimized::{global::MeasureIndex, note::Finger, track::MeasureData},
};

use crate::loader::load_song;

/// Compute and display left-hand guitar fingering for tab tracks
#[derive(Bpaf, Debug)]
#[bpaf(command("fingering"))]
pub struct FingeringArgs {
    /// Input Guitar Pro file (.gp3/.gp4/.gp5/.gp/.gpx)
    #[bpaf(short, long, argument("PATH"))]
    pub input: String,

    /// Filter by track name (case-insensitive substring match)
    #[bpaf(long, argument("NAME"))]
    pub track: Option<String>,

    /// Output JSON instead of human-readable text
    #[bpaf(long, switch)]
    pub json: bool,

    /// Persist finger assignments into the optimized model and write to this path (.score JSON)
    #[bpaf(long, argument("PATH"))]
    pub annotate: Option<String>,

    /// Force starting hand position (fret number; not yet implemented in the underlying algorithm)
    // parsed for forward-compat; not yet wired into the fingering algorithm
    #[bpaf(long, argument("FRET"))]
    #[allow(dead_code)]
    pub position: Option<u8>,
}

// ─── JSON output types ──────────────────────────────────────────────────────

#[derive(Serialize)]
struct JsonOutput {
    file: String,
    tracks: Vec<JsonTrack>,
}

#[derive(Serialize)]
struct JsonTrack {
    name: String,
    measures: Vec<JsonMeasure>,
}

#[derive(Serialize)]
struct JsonMeasure {
    measure: u16, // 1-based
    assignments: Vec<JsonAssignment>,
}

#[derive(Serialize)]
struct JsonAssignment {
    string: i8,
    fret: i16,
    finger: u8,
    role: &'static str,
    position_shift: bool,
}

// ─── Entry point ────────────────────────────────────────────────────────────

pub fn run(args: &FingeringArgs) -> anyhow::Result<()> {
    let (song, _fmt) = load_song(&args.input)?;
    let loaded = legacy_song_to_loaded_score(&song);

    let filter = args.track.as_deref().map(|s| s.to_lowercase());

    let matching_indices: Vec<usize> = loaded
        .score
        .tracks
        .iter()
        .enumerate()
        .filter(|(_, t)| {
            filter
                .as_deref()
                .is_none_or(|f| t.name.to_lowercase().contains(f))
        })
        .map(|(i, _)| i)
        .collect();

    if matching_indices.is_empty() {
        anyhow::bail!("No tracks match the filter");
    }

    if args.json {
        let mut out = JsonOutput {
            file: args.input.clone(),
            tracks: Vec::new(),
        };
        for &idx in &matching_indices {
            let track = &loaded.score.tracks[idx];
            let measures: Vec<&MeasureData> = track.measures.values().collect();
            let all_assignments = suggest_fingering(&measures, &[]);
            out.tracks.push(build_json_track(
                &track.name,
                &track.measures,
                &all_assignments,
            ));
        }
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    // ── Human-readable output ────────────────────────────────────────────────
    for &idx in &matching_indices {
        let track = &loaded.score.tracks[idx];
        let measures: Vec<&MeasureData> = track.measures.values().collect();
        let all_assignments = suggest_fingering(&measures, &[]);

        println!("Track: {}", track.name);
        for (mdata, measure_assignments) in track.measures.values().zip(all_assignments.iter()) {
            if measure_assignments.is_empty() {
                continue;
            }
            print_measure(mdata.measure_index.0 + 1, measure_assignments);
        }
        println!();
    }

    // ── --annotate: persist assignments and write .score JSON ────────────────
    if let Some(out_path) = &args.annotate {
        // Re-convert so we can mutate freely
        let mut loaded_mut = legacy_song_to_loaded_score(&song);

        for &idx in &matching_indices {
            let track = &loaded_mut.score.tracks[idx];
            // Collect assignments with immutable borrow first
            let measures: Vec<&MeasureData> = track.measures.values().collect();
            let all_assignments = suggest_fingering(&measures, &[]);
            // Build lookup per measure_index
            let assignment_map: HashMap<MeasureIndex, &[FingerAssignment]> = track
                .measures
                .keys()
                .copied()
                .zip(all_assignments.iter().map(|v| v.as_slice()))
                .collect();

            // Now mutate
            for (midx, mdata) in loaded_mut.score.tracks[idx].measures.iter_mut() {
                if let Some(assignments) = assignment_map.get(midx) {
                    apply_fingers(mdata, assignments);
                }
            }
        }

        let json = serde_json::to_string_pretty(&loaded_mut.score)
            .context("Failed to serialize annotated score")?;
        std::fs::write(out_path, json).with_context(|| format!("Failed to write '{out_path}'"))?;
        eprintln!("Wrote annotated score to '{out_path}'");
    }

    Ok(())
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn build_json_track(
    name: &str,
    measure_map: &std::collections::BTreeMap<MeasureIndex, MeasureData>,
    all_assignments: &[Vec<FingerAssignment>],
) -> JsonTrack {
    let mut measures = Vec::new();
    for (mdata, measure_assignments) in measure_map.values().zip(all_assignments.iter()) {
        if measure_assignments.is_empty() {
            continue;
        }
        measures.push(JsonMeasure {
            measure: mdata.measure_index.0 + 1,
            assignments: measure_assignments
                .iter()
                .map(|a| JsonAssignment {
                    string: a.string,
                    fret: a.fret,
                    finger: a.finger,
                    role: role_str(a.role),
                    position_shift: a.position_shift,
                })
                .collect(),
        });
    }
    JsonTrack {
        name: name.to_owned(),
        measures,
    }
}

fn role_str(role: FingerRole) -> &'static str {
    match role {
        FingerRole::Single => "single",
        FingerRole::BarreAnchor => "barre_anchor",
        FingerRole::BarreMember => "barre_member",
    }
}

fn print_measure(measure_number: u16, assignments: &[FingerAssignment]) {
    let has_shift = assignments.iter().any(|a| a.position_shift);
    let suffix = if has_shift { "  [position shift]" } else { "" };
    println!("  Measure {measure_number}:{suffix}");

    // Collect all strings present, sorted
    let mut strings: Vec<i8> = assignments.iter().map(|a| a.string).collect();
    strings.sort_unstable();
    strings.dedup();

    for s in strings {
        let notes: Vec<_> = assignments.iter().filter(|a| a.string == s).collect();
        let pairs: Vec<String> = notes
            .iter()
            .map(|a| {
                let role_tag = match a.role {
                    FingerRole::BarreAnchor => " (barre)",
                    FingerRole::BarreMember => " (barre)",
                    FingerRole::Single => "",
                };
                format!("fret {:2} → {}{}", a.fret, finger_name(a.finger), role_tag)
            })
            .collect();
        println!("    str {s}: {}", pairs.join("  |  "));
    }
}

fn finger_name(f: u8) -> &'static str {
    match f {
        1 => "index",
        2 => "middle",
        3 => "ring",
        4 => "pinky",
        _ => "?",
    }
}

/// Write computed finger assignments into `Note.left_finger` for matching string+fret.
fn apply_fingers(measure: &mut MeasureData, assignments: &[FingerAssignment]) {
    // Build (string, fret) → Finger lookup
    let lookup: HashMap<(i8, i16), Finger> = assignments
        .iter()
        .map(|a| ((a.string, a.fret), finger_from_u8(a.finger)))
        .collect();

    for voice in measure.voices.values_mut() {
        for beat in &mut voice.beats {
            for note in &mut beat.notes {
                if let (Some(s), Some(f)) = (note.string, note.fret) {
                    let key = (s as i8, f as i16);
                    if let Some(&finger) = lookup.get(&key) {
                        note.left_finger = Some(finger);
                    }
                }
            }
        }
    }
}

fn finger_from_u8(f: u8) -> Finger {
    match f {
        1 => Finger::Index,
        2 => Finger::Middle,
        3 => Finger::Ring,
        4 => Finger::Pinky,
        _ => Finger::Open,
    }
}
