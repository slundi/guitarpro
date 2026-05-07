//! Conversion from `legacy::Song` to `musicxml::ScorePartwise`.
//!
//! Entry point: [`song_to_score_partwise`].

use crate::model::{
    legacy::{
        key_signature::{Duration, DURATION_QUARTER_TIME},
        song::Song,
    },
    musicxml::{
        self,
        note::NoteTypeValue,
    },
};

/// Divisions per quarter note used throughout the output document.
///
/// Matches `DURATION_QUARTER_TIME` so legacy tick values map 1-to-1.
pub const DIVISIONS: u32 = DURATION_QUARTER_TIME as u32;

// ---------------------------------------------------------------------------
// Pitch helpers
// ---------------------------------------------------------------------------

/// Convert a MIDI note number to a MusicXML pitch triple `(step, alter, octave)`.
///
/// Uses sharps for accidentals (alter = 1.0).
/// MIDI 60 = C4 (middle C).
pub fn midi_to_pitch(midi: i8) -> (String, Option<f64>, i8) {
    // MIDI octave: octave = (midi / 12) - 1, but we need signed-safe arithmetic
    let midi_u = midi as i32;
    let pitch_class = midi_u.rem_euclid(12) as u8;
    let octave = (midi_u / 12 - 1) as i8;

    let (step, alter) = match pitch_class {
        0  => ("C", None),
        1  => ("C", Some(1.0)),
        2  => ("D", None),
        3  => ("D", Some(1.0)),
        4  => ("E", None),
        5  => ("F", None),
        6  => ("F", Some(1.0)),
        7  => ("G", None),
        8  => ("G", Some(1.0)),
        9  => ("A", None),
        10 => ("A", Some(1.0)),
        11 => ("B", None),
        _  => unreachable!(),
    };

    (step.to_string(), alter, octave)
}

// ---------------------------------------------------------------------------
// Duration helpers
// ---------------------------------------------------------------------------

/// Convert a legacy [`Duration`] to MusicXML `<divisions>` ticks.
///
/// Result is expressed in units of [`DIVISIONS`] per quarter note.
/// Dotted and tuplet adjustments are applied.
pub fn duration_to_divisions(d: &Duration) -> u32 {
    // Base ticks for this note value (quarter = DIVISIONS)
    let base = (DIVISIONS * 4) / d.value as u32;

    // Dotted: multiply by 3/2
    let dotted = if d.dotted {
        base + base / 2
    } else {
        base
    };

    // Tuplet: scale by tuplet_times / tuplet_enters
    if d.tuplet_enters != 1 || d.tuplet_times != 1 {
        dotted * d.tuplet_times as u32 / d.tuplet_enters as u32
    } else {
        dotted
    }
}

/// Convert a legacy [`Duration`] value to the MusicXML [`NoteTypeValue`] symbol.
pub fn duration_to_note_type(d: &Duration) -> NoteTypeValue {
    match d.value {
        1   => NoteTypeValue::Whole,
        2   => NoteTypeValue::Half,
        4   => NoteTypeValue::Quarter,
        8   => NoteTypeValue::Eighth,
        16  => NoteTypeValue::N16th,
        32  => NoteTypeValue::N32nd,
        64  => NoteTypeValue::N64th,
        128 => NoteTypeValue::N128th,
        _   => NoteTypeValue::Quarter, // fallback
    }
}

// ---------------------------------------------------------------------------
// Top-level conversion
// ---------------------------------------------------------------------------

/// Convert a legacy [`Song`] into a MusicXML [`ScorePartwise`] document.
///
/// The conversion is organized in passes:
/// 1. Score metadata (work title, identification)
/// 2. Part list (one `ScorePart` per track)
/// 3. Parts (measures → notes, per track)
pub fn song_to_score_partwise(song: &Song) -> musicxml::ScorePartwise {
    let identification = build_identification(song);
    let part_list = build_part_list(song);
    let parts = build_parts(song);

    musicxml::ScorePartwise {
        version: Some("4.0".to_string()),
        work: Some(musicxml::Work {
            work_number: None,
            work_title: Some(song.name.clone()).filter(|s| !s.is_empty()),
            opus: None,
        }),
        movement_number: None,
        movement_title: None,
        identification: Some(identification),
        defaults: None,
        credits: vec![],
        part_list,
        parts,
    }
}

// ---------------------------------------------------------------------------
// Stubs — filled in subsequent commits
// ---------------------------------------------------------------------------

fn build_identification(song: &Song) -> musicxml::identification::Identification {
    use musicxml::identification::{Creator, Encoding, Identification, Rights, Supports};

    let mut creators = vec![];
    if !song.artist.is_empty() {
        creators.push(Creator {
            creator_type: Some("composer".to_string()),
            value: song.artist.clone(),
        });
    }
    if !song.author.is_empty() {
        creators.push(Creator {
            creator_type: Some("arranger".to_string()),
            value: song.author.clone(),
        });
    }
    if !song.words.is_empty() && song.words != song.author {
        creators.push(Creator {
            creator_type: Some("lyricist".to_string()),
            value: song.words.clone(),
        });
    }
    if !song.writer.is_empty() {
        creators.push(Creator {
            creator_type: Some("transcriber".to_string()),
            value: song.writer.clone(),
        });
    }

    let rights = if !song.copyright.is_empty() {
        vec![Rights {
            rights_type: None,
            value: song.copyright.clone(),
        }]
    } else {
        vec![]
    };

    let encoding = Some(Encoding {
        encoding_date: None,
        encoders: vec![],
        software: vec!["guitarpro (Rust)".to_string()],
        encoding_description: vec![],
        supports: vec![
            Supports {
                supports_type: "yes".to_string(),
                element: "accidental".to_string(),
                attribute: None,
                value: None,
            },
            Supports {
                supports_type: "yes".to_string(),
                element: "beam".to_string(),
                attribute: None,
                value: None,
            },
        ],
    });

    // Collect remaining free-text metadata into miscellaneous fields
    let mut misc_fields = vec![];
    if !song.subtitle.is_empty() {
        misc_fields.push(musicxml::identification::MiscellaneousField {
            name: "subtitle".to_string(),
            value: song.subtitle.clone(),
        });
    }
    if !song.album.is_empty() {
        misc_fields.push(musicxml::identification::MiscellaneousField {
            name: "album".to_string(),
            value: song.album.clone(),
        });
    }
    if !song.date.is_empty() {
        misc_fields.push(musicxml::identification::MiscellaneousField {
            name: "date".to_string(),
            value: song.date.clone(),
        });
    }
    if !song.instructions.is_empty() {
        misc_fields.push(musicxml::identification::MiscellaneousField {
            name: "instructions".to_string(),
            value: song.instructions.clone(),
        });
    }
    for (i, notice) in song.notice.iter().enumerate() {
        misc_fields.push(musicxml::identification::MiscellaneousField {
            name: format!("notice-{}", i + 1),
            value: notice.clone(),
        });
    }

    let miscellaneous = if misc_fields.is_empty() {
        None
    } else {
        Some(musicxml::identification::Miscellaneous { fields: misc_fields })
    };

    Identification {
        creators,
        rights,
        encoding,
        source: None,
        relations: vec![],
        miscellaneous,
    }
}

fn build_part_list(_song: &Song) -> musicxml::part_list::PartList {
    musicxml::part_list::PartList { items: vec![] }
}

fn build_parts(_song: &Song) -> Vec<musicxml::Part> {
    vec![]
}
