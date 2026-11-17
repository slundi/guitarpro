use super::common::read_file;
use crate::error::GpResult;
use crate::model::legacy::headers::{EndingWalks, MeasureHeader};
use crate::model::legacy::song::Song;

#[test]
fn test_gp3_duration() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/Duration.gp3")))
        .unwrap();
}

#[test]
fn test_gp4_key() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/Key.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_key() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Key.gp5")))
        .unwrap();
}

#[test]
fn test_gp4_keysig() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/keysig.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_keysig() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/keysig.gp5")))
        .unwrap();
}

#[test]
fn test_gp4_repeat() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/Repeat.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_repeat() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Repeat.gp5")))
        .unwrap();
}

#[test]
fn test_gp5_voices() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Voices.gp5")))
        .unwrap();
}

#[test]
fn test_gp5_dotted_tuplets() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/dotted-tuplets.gp5")))
        .unwrap();
}
#[test]
fn test_gp4_test_irr_tuplet() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/testIrrTuplet.gp4")))
        .unwrap();
}
#[test]
fn test_gp4_tuplet_with_slur() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/tuplet-with-slur.gp4")))
        .unwrap();
}

#[test]
fn test_gp4_rest_centered() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/rest-centered.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_rest_centered() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/rest-centered.gp5")))
        .unwrap();
}

#[test]
fn test_gp5_rse() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/RSE.gp5")))
        .unwrap();
}

#[test]
fn test_gp3_volta() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/volta.gp3")))
        .unwrap();
}
#[test]
fn test_gp4_volta() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/volta.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_volta() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/volta.gp5")))
        .unwrap();
}

/// The mask [`EndingWalks`] reads `ordinal` as after
/// measures with `previous` masks (the first opening the repeat), the last
/// closing it if `closed`.
fn alternative_after(previous: &[u8], closed: bool, ordinal: u8) -> GpResult<u8> {
    let last = previous.len().saturating_sub(1);
    let song = Song {
        measure_headers: previous
            .iter()
            .enumerate()
            .map(|(index, &repeat_alternative)| MeasureHeader {
                repeat_open: index == 0,
                repeat_alternative,
                repeat_close: if closed && index == last { 1 } else { -1 },
                ..MeasureHeader::default()
            })
            .collect(),
        ..Song::default()
    };
    Ok(EndingWalks::default().passes(&song.measure_headers, ordinal, false))
}

#[test]
fn a_restated_ending_ordinal_continues_its_ending() -> GpResult<()> {
    // Ending 2 over two measures: the second restates its ordinal.
    assert_eq!(alternative_after(&[0, 1, 2], false, 2)?, 2);
    Ok(())
}

#[test]
fn a_lower_ordinal_does_not_continue_an_ending() -> GpResult<()> {
    // An ending for passes 1 and 2 (ordinal 2), then ordinal 1: no pass
    // is left for it, and it isn't the ending's.
    assert_eq!(alternative_after(&[0, 0b011], false, 1)?, 0);
    Ok(())
}

#[test]
fn an_ordinal_outside_the_last_ending_does_not_continue_it() -> GpResult<()> {
    // Endings 2 and 1, then 2 restated after ending 1.
    assert_eq!(alternative_after(&[0, 2, 1], false, 2)?, 0);
    Ok(())
}

#[test]
fn an_ordinal_after_the_repeats_close_does_not_continue_its_ending() -> GpResult<()> {
    // Ending 2 closes the repeat; a new bracket follows.
    assert_eq!(alternative_after(&[0, 1, 2], true, 2)?, 0);
    Ok(())
}

#[test]
fn an_ending_on_a_repeats_start_does_not_continue_the_ending_before() -> GpResult<()> {
    // A repeat's last ending (2), then a measure opening the next repeat
    // with an ending of ordinal 2: its own, for passes 1 and 2.
    let song = Song {
        measure_headers: [(0, true, -1), (1, false, 1), (2, false, -1)]
            .iter()
            .map(
                |&(repeat_alternative, repeat_open, repeat_close)| MeasureHeader {
                    repeat_open,
                    repeat_alternative,
                    repeat_close,
                    ..MeasureHeader::default()
                },
            )
            .collect(),
        ..Song::default()
    };
    assert_eq!(
        EndingWalks::default().passes(&song.measure_headers, 2, false),
        2
    );
    assert_eq!(
        EndingWalks::default().passes(&song.measure_headers, 2, true),
        0b11
    );
    Ok(())
}

#[test]
fn an_earlier_ordinal_restated_after_a_later_ending_is_in_none() -> GpResult<()> {
    // Endings 1 and 2, then 1 again: no pass is left for it.
    assert_eq!(alternative_after(&[0, 1, 2], false, 1)?, 0);
    Ok(())
}

/// A second repeat without a forward barline has endings of its own: the
/// first repeat's, before its body, don't take its passes.
#[test]
fn a_repeat_without_a_start_has_its_own_endings() -> GpResult<()> {
    // Body, ending 1 (closing), ending 2; body again: (mask, closes).
    let measures = [(0, false), (1, true), (2, false), (0, false)];
    let mut song = Song {
        measure_headers: measures
            .iter()
            .map(|&(repeat_alternative, closes)| MeasureHeader {
                repeat_alternative,
                repeat_close: if closes { 1 } else { -1 },
                ..MeasureHeader::default()
            })
            .collect(),
        ..Song::default()
    };
    let first = EndingWalks::default().passes(&song.measure_headers, 1, false);
    assert_eq!(first, 1);
    song.measure_headers.push(MeasureHeader {
        repeat_alternative: first,
        repeat_close: 1,
        ..MeasureHeader::default()
    });
    assert_eq!(
        EndingWalks::default().passes(&song.measure_headers, 2, false),
        2
    );
    Ok(())
}

/// An ending after a measure in none is still the repeat's next one when
/// the ending before that measure closed the repeat: `|: A |1 B :| C |2 D`.
#[test]
fn an_ending_after_a_measure_in_none_continues_its_repeat() -> GpResult<()> {
    // A (opening), ending 1 (closing), C: D's ending 2 is just pass 2.
    let measures = [(true, 0, false), (false, 1, true), (false, 0, false)];
    let song = Song {
        measure_headers: measures
            .iter()
            .map(|&(repeat_open, repeat_alternative, closes)| MeasureHeader {
                repeat_open,
                repeat_alternative,
                repeat_close: if closes { 1 } else { -1 },
                ..MeasureHeader::default()
            })
            .collect(),
        ..Song::default()
    };
    assert_eq!(
        EndingWalks::default().passes(&song.measure_headers, 2, false),
        2
    );
    Ok(())
}

/// A repeat without a forward barline right after another's last ending
/// starts its own endings: `X |1 Y :| 2 Z |1 V`.
#[test]
fn endings_right_after_a_last_ending_start_anew() -> GpResult<()> {
    // X, ending 1 (closing), ending 2: V's ending 1 is the next repeat's.
    let measures = [(0, false), (1, true), (2, false)];
    let song = Song {
        measure_headers: measures
            .iter()
            .map(|&(repeat_alternative, closes)| MeasureHeader {
                repeat_alternative,
                repeat_close: if closes { 1 } else { -1 },
                ..MeasureHeader::default()
            })
            .collect(),
        ..Song::default()
    };
    assert_eq!(
        EndingWalks::default().passes(&song.measure_headers, 1, false),
        1
    );
    Ok(())
}

/// A multi-measure last ending continues past its first measure.
#[test]
fn a_multi_measure_last_ending_continues() -> GpResult<()> {
    // |: A |1 B :| 2 C1, then C2 restating 2.
    let measures = [(true, 0, false), (false, 1, true), (false, 2, false)];
    let song = Song {
        measure_headers: measures
            .iter()
            .map(|&(repeat_open, repeat_alternative, closes)| MeasureHeader {
                repeat_open,
                repeat_alternative,
                repeat_close: if closes { 1 } else { -1 },
                ..MeasureHeader::default()
            })
            .collect(),
        ..Song::default()
    };
    assert_eq!(
        EndingWalks::default().passes(&song.measure_headers, 2, false),
        2
    );
    Ok(())
}

/// A tied note takes the fret of the last note before it on its string,
/// in the same measure too (GP writes no fret for it; ours writes 0).
#[test]
fn test_gp5_tied_note_takes_the_last_fret_on_its_string() -> GpResult<()> {
    use crate::model::legacy::beat::Beat;
    use crate::model::legacy::enums::NoteType;
    use crate::model::legacy::note::Note;
    let mut song = Song::default();
    song.read_gp5(&read_file(String::from("test/Key.gp5")))?;
    let note = |value, kind| Note {
        string: 2,
        value,
        kind,
        ..Default::default()
    };
    let beat = |notes| Beat {
        notes,
        ..Default::default()
    };
    let voice = &mut song.tracks[0].measures[0].voices[0];
    voice.beats = vec![
        beat(vec![note(3, NoteType::Normal)]),
        beat(vec![note(5, NoteType::Normal)]),
        beat(vec![note(0, NoteType::Tie)]),
    ];
    let mut read = Song::default();
    read.read_gp5(&song.write((5, 1, 0), None)?)?;
    let values: Vec<i16> = read.tracks[0].measures[0].voices[0]
        .beats
        .iter()
        .flat_map(|b| b.notes.iter().map(|n| n.value))
        .collect();
    assert_eq!(values, [3, 5, 5]);
    Ok(())
}

/// GP5 keeps each direction sign at one measure: one in several is written
/// at its first, where playback reaches it first.
#[test]
fn test_gp5_sign_in_several_measures_keeps_its_first() -> GpResult<()> {
    use crate::model::legacy::enums::DirectionSign;
    let mut song = Song::default();
    song.read_gp5(&read_file(String::from("test/volta.gp5")))?;
    assert!(song.measure_headers.len() >= 2);
    for header in &mut song.measure_headers {
        header.directions.clear();
    }
    song.measure_headers[0].directions = vec![DirectionSign::DaCoda];
    song.measure_headers[1].directions = vec![DirectionSign::DaCoda];
    let mut read = Song::default();
    read.read_gp5(&song.write(song.version.number, None)?)?;
    let at: Vec<usize> = read
        .measure_headers
        .iter()
        .enumerate()
        .filter(|(_, header)| header.directions.contains(&DirectionSign::DaCoda))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(at, [0]);
    Ok(())
}

/// A measure's one sign, as each format's reader took it before measures
/// kept all of theirs: GPIF a target over a jump, GP3–5 a jump over a target.
#[test]
fn test_direction_label_follows_the_formats_reader() {
    use crate::analysis::repeats::direction_label;
    use crate::model::legacy::enums::DirectionSign;
    let mut header = MeasureHeader::default();
    assert_eq!(direction_label(&header, (7, 0, 0)), None);
    header.add_directions([DirectionSign::DaCapoAlCoda, DirectionSign::Coda]);
    assert_eq!(direction_label(&header, (7, 0, 0)).as_deref(), Some("Coda"));
    assert_eq!(
        direction_label(&header, (5, 1, 0)).as_deref(),
        Some("D.C. al Coda")
    );
    // D.C. labelled as in `directions`.
    let mut header = MeasureHeader::default();
    header.add_directions([DirectionSign::DaCapo]);
    assert_eq!(direction_label(&header, (5, 1, 0)).as_deref(), Some("D.C."));
    assert_eq!(header.direction_labels(), ["D.C."]);
}

/// A D.S.S. or double-coda jump, which the play sequence can't follow, is
/// flagged; the jumps it follows and the signs it jumps to aren't.
#[test]
fn test_unfollowed_jumps_are_flagged() {
    use crate::analysis::repeats::has_unfollowed_jumps;
    use crate::model::legacy::enums::DirectionSign;
    let headers = |signs: &[DirectionSign]| {
        let mut header = MeasureHeader::default();
        header.add_directions(signs.iter().copied());
        vec![header]
    };
    assert!(!has_unfollowed_jumps(&headers(&[
        DirectionSign::Segno,
        DirectionSign::SegnoSegno,
        DirectionSign::DoubleCoda,
        DirectionSign::DaSegnoAlCoda,
    ])));
    assert!(has_unfollowed_jumps(&headers(&[
        DirectionSign::DaSegnoSegno
    ])));
    assert!(has_unfollowed_jumps(&headers(&[
        DirectionSign::DaCapoAlDoubleCoda
    ])));
}

/// Each ending walks back only to the measure in none before it, whose
/// walk on is kept: a long file of endings reads in linear time, giving
/// what a walk back to the repeat's start gives.
#[test]
fn many_endings_read_without_rewalking_the_file() -> GpResult<()> {
    // Implicit repeats each with a first ending closing it: after the
    // first, ordinal 1 has no pass left (the endings all close this
    // repeat, sending it round again).
    let mut song = Song::default();
    let mut walks = EndingWalks::default();
    for index in 0..40_000 {
        let ending = index % 2 == 1;
        let repeat_alternative = if ending {
            walks.passes(&song.measure_headers, 1, false)
        } else {
            0
        };
        assert_eq!(repeat_alternative, u8::from(index == 1));
        song.measure_headers.push(MeasureHeader {
            repeat_alternative,
            repeat_close: if ending { 1 } else { -1 },
            ..MeasureHeader::default()
        });
    }
    Ok(())
}

/// GP3/4 store an ending as one ordinal: passes 1 and 3 before a 2nd ending
/// don't survive (pass 3 is lost), passes 1 and 2 do; GP5 keeps the mask.
#[test]
fn write_losses_report_endings_gp3_4_cannot_store() {
    use crate::model::legacy::enums::DirectionSign;
    let header = |repeat_open, repeat_alternative, repeat_close| MeasureHeader {
        repeat_open,
        repeat_alternative,
        repeat_close,
        ..MeasureHeader::default()
    };
    let lossy = Song {
        measure_headers: vec![
            header(true, 0, -1),
            header(false, 0b101, 2),
            header(false, 0b010, -1),
        ],
        ..Song::default()
    };
    let name = "Ending passes (GP3/4 store one ordinal per ending)";
    assert_eq!(lossy.write_losses((4, 0, 6)).get(name), 1);
    assert!(lossy.write_losses((5, 1, 0)).is_empty());
    let kept = Song {
        measure_headers: vec![
            header(true, 0, -1),
            header(false, 0b011, 2),
            header(false, 0b100, -1),
        ],
        ..Song::default()
    };
    assert!(kept.write_losses((4, 0, 6)).is_empty());

    // Two Fines: GP5 keeps one; GP3/4 none.
    let mut fines = Song {
        measure_headers: vec![MeasureHeader::default(), MeasureHeader::default()],
        ..Song::default()
    };
    for header in &mut fines.measure_headers {
        header.directions.push(DirectionSign::Fine);
    }
    assert_eq!(
        fines
            .write_losses((5, 1, 0))
            .get("Direction sign in another measure (GP5 keeps one per sign)"),
        1
    );
    assert_eq!(
        fines
            .write_losses((3, 0, 0))
            .get("Direction sign (GP3/4 have none)"),
        2
    );
}

/// `|: [1. :| [2.`: an ending in the measure opening the repeat is its
/// own, so the 2nd ending is pass 2 alone.
#[test]
fn an_ending_in_the_opening_measure_is_the_repeats() -> GpResult<()> {
    assert_eq!(alternative_after(&[1], true, 2)?, 2);
    Ok(())
}

/// A last ending that opens the next repeat is the earlier repeat's: the
/// next repeat's 1st ending is pass 1.
#[test]
fn a_last_ending_opening_a_repeat_is_the_earlier_repeats() -> GpResult<()> {
    let header = |repeat_open, repeat_alternative, repeat_close| MeasureHeader {
        repeat_open,
        repeat_alternative,
        repeat_close,
        ..MeasureHeader::default()
    };
    let song = Song {
        measure_headers: vec![
            header(true, 0, -1),
            header(false, 1, 1),
            header(true, 2, -1),
            header(false, 0, -1),
        ],
        ..Song::default()
    };
    assert_eq!(
        EndingWalks::default().passes(&song.measure_headers, 1, false),
        1
    );
    Ok(())
}

/// An ending's GP3/4 ordinal: the last pass of its first run of passes.
#[test]
fn ending_ordinals_are_the_last_pass_of_the_first_run() {
    use crate::model::legacy::headers::ending_ordinal;
    for (mask, ordinal) in [
        (0, 0),
        (0b1, 1),
        (0b11, 2),
        (0b101, 1),
        (0b110, 3),
        (0b1000_0000, 8),
        (0xFF, 8),
    ] {
        assert_eq!(ending_ordinal(mask), ordinal, "{mask:#b}");
    }
}
