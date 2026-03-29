use crate::model::song::Song;
use fraction::ToPrimitive;
use std::{fs, io::Read};

fn read_file(path: String) -> Vec<u8> {
    let test_path = if path.starts_with("test/") {
        format!("../{}", path)
    } else {
        format!("../test/{}", path)
    };
    let f = fs::OpenOptions::new()
        .read(true)
        .open(&test_path)
        .unwrap_or_else(|e| panic!("Cannot open file '{}': {}", test_path, e));
    let size: usize = fs::metadata(&test_path)
        .unwrap_or_else(|e| panic!("Unable to get file size for '{}': {}", test_path, e))
        .len()
        .to_usize()
        .unwrap();
    let mut data: Vec<u8> = Vec::with_capacity(size);
    f.take(size as u64)
        .read_to_end(&mut data)
        .unwrap_or_else(|e| panic!("Unable to read file contents from '{}': {}", test_path, e));
    data
}

#[test]
fn test_gp3_chord() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/Chords.gp3")))
        .unwrap();
}
#[test]
fn test_gp4_chord() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/Chords.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_chord() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Chords.gp5")))
        .unwrap();
}
#[test]
fn test_gp5_unknown_chord_extension() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Unknown Chord Extension.gp5")))
        .unwrap();
}
#[test]
fn test_gp5_chord_without_notes() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/chord_without_notes.gp5")))
        .unwrap();
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/001_Funky_Guy.gp5")))
        .unwrap();
}

#[test]
fn test_gp3_duration() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/Duration.gp3")))
        .unwrap();
}

#[test]
fn test_gp3_effects() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/Effects.gp3")))
        .unwrap();
}
#[test]
fn test_gp4_effects() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/Effects.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_effects() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Effects.gp5")))
        .unwrap();
}

#[test]
fn test_gp3_harmonics() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/Harmonics.gp3")))
        .unwrap();
}
#[test]
fn test_gp4_harmonics() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/Harmonics.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_harmonics() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Harmonics.gp5")))
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
fn test_gp5_rse() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/RSE.gp5")))
        .unwrap();
}

#[test]
fn test_gp4_slides() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/Slides.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_slides() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Slides.gp5")))
        .unwrap();
}

#[test]
fn test_gp4_strokes() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/Strokes.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_strokes() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Strokes.gp5")))
        .unwrap();
}

#[test]
fn test_gp4_vibrato() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/Vibrato.gp4")))
        .unwrap();
}

#[test]
fn test_gp5_voices() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Voices.gp5")))
        .unwrap();
}

#[test]
fn test_gp5_no_wah() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/No Wah.gp5")))
        .unwrap();
}
#[test]
fn test_gp5_wah() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Wah.gp5")))
        .unwrap();
}
#[test]
fn test_gp5_wah_m() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/Wah-m.gp5")))
        .unwrap();
}

#[test]
fn test_gp5_all_percussion() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/all-percussion.gp5")))
        .unwrap();
}
#[test]
fn test_gp5_basic_bend() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/basic-bend.gp5")))
        .unwrap();
}
#[test]
fn test_gp5_beams_sterms_ledger_lines() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from(
        "test/beams-stems-ledger-lines.gp5",
    )))
    .unwrap();
}
#[test]
fn test_gp5_brush() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/brush.gp5")))
        .unwrap();
}
#[test]
fn test_gp3_capo_fret() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/capo-fret.gp3")))
        .unwrap();
}
#[test]
fn test_gp4_capo_fret() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/capo-fret.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_capo_fret() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/capo-fret.gp5")))
        .unwrap();
}
#[test]
fn test_gp3_copyright() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/copyright.gp3")))
        .unwrap();
}
#[test]
fn test_gp4_copyright() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/copyright.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_copyright() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/copyright.gp5")))
        .unwrap();
}
#[test]
fn test_gp3_dotted_gliss() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/dotted-gliss.gp3")))
        .unwrap();
}
#[test]
fn test_gp5_dotted_tuplets() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/dotted-tuplets.gp5")))
        .unwrap();
}
#[test]
fn test_gp5_dynamic() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/dynamic.gp5")))
        .unwrap();
}
#[test]
fn test_gp4_fade_in() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/fade-in.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_fade_in() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/fade-in.gp5")))
        .unwrap();
}
#[test]
fn test_gp4_fingering() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/fingering.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_fingering() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/fingering.gp5")))
        .unwrap();
}
#[test]
fn test_gp4_fret_diagram() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/fret-diagram.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_fret_diagram() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/fret-diagram.gp5")))
        .unwrap();
}
#[test]
fn test_gp3_ghost_note() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/ghost_note.gp3")))
        .unwrap();
}
#[test]
fn test_gp5_grace() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/grace.gp5")))
        .unwrap();
}
#[test]
fn test_gp5_heavy_accent() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/heavy-accent.gp5")))
        .unwrap();
}
#[test]
fn test_gp3_high_pitch() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/high-pitch.gp3")))
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
fn test_gp4_legato_slide() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/legato-slide.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_legato_slide() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/legato-slide.gp5")))
        .unwrap();
}
#[test]
fn test_gp4_let_ring() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/let-ring.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_let_ring() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/let-ring.gp5")))
        .unwrap();
}
#[test]
fn test_gp4_palm_mute() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/palm-mute.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_palm_mute() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/palm-mute.gp5")))
        .unwrap();
}
#[test]
fn test_gp4_pick_up_down() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/pick-up-down.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_pick_up_down() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/pick-up-down.gp5")))
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
fn test_gp4_sforzato() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/sforzato.gp4")))
        .unwrap();
}
#[test]
fn test_gp4_shift_slide() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/shift-slide.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_shift_slide() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/shift-slide.gp5")))
        .unwrap();
}
#[test]
fn test_gp4_slide_in_above() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/slide-in-above.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_slide_in_above() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/slide-in-above.gp5")))
        .unwrap();
}
#[test]
fn test_gp4_slide_in_below() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/slide-in-below.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_slide_in_below() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/slide-in-below.gp5")))
        .unwrap();
}
#[test]
fn test_gp4_slide_out_down() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/slide-out-down.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_slide_out_down() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/slide-out-down.gp5")))
        .unwrap();
}
#[test]
fn test_gp4_slide_out_up() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/slide-out-up.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_slide_out_up() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/slide-out-up.gp5")))
        .unwrap();
}
#[test]
fn test_gp4_slur() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/slur.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_slur_notes_effect_mask() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/slur-notes-effect-mask.gp5")))
        .unwrap();
}
#[test]
fn test_gp5_tap_slap_pop() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/tap-slap-pop.gp5")))
        .unwrap();
}
#[test]
fn test_gp3_tempo() {
    let mut song: Song = Song::default();
    song.read_gp3(&read_file(String::from("test/tempo.gp3")))
        .unwrap();
}
#[test]
fn test_gp4_tempo() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/tempo.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_tempo() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/tempo.gp5")))
        .unwrap();
}
#[test]
fn test_gp4_test_irr_tuplet() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/testIrrTuplet.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_tremolos() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/tremolos.gp5")))
        .unwrap();
}
#[test]
fn test_gp4_trill() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/trill.gp4")))
        .unwrap();
}
#[test]
fn test_gp4_tuplet_with_slur() {
    let mut song: Song = Song::default();
    song.read_gp4(&read_file(String::from("test/tuplet-with-slur.gp4")))
        .unwrap();
}
#[test]
fn test_gp5_vibrato() {
    let mut song: Song = Song::default();
    song.read_gp5(&read_file(String::from("test/vibrato.gp5")))
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

// ==================== GPX (Guitar Pro 6) tests ====================

fn read_gpx(filename: &str) -> Song {
    let mut song = Song::default();
    song.read_gpx(&read_file(String::from(filename))).unwrap();
    song
}

#[test]
fn test_gpx_keysig() {
    let song = read_gpx("test/keysig.gpx");
    assert_eq!(song.tracks.len(), 1);
    assert_eq!(song.measure_headers.len(), 32);
}
#[test]
fn test_gpx_copyright() {
    let song = read_gpx("test/copyright.gpx");
    assert!(!song.tracks.is_empty());
    assert!(
        !song.copyright.is_empty(),
        "copyright field should be populated"
    );
}
#[test]
fn test_gpx_tempo() {
    let song = read_gpx("test/tempo.gpx");
    assert!(!song.measure_headers.is_empty());
    assert!(song.tempo > 0, "tempo should be parsed from automations");
}
#[test]
fn test_gpx_rest_centered() {
    let song = read_gpx("test/rest-centered.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_dotted_tuplets() {
    let song = read_gpx("test/dotted-tuplets.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_tuplets() {
    let song = read_gpx("test/tuplets.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_tuplets2() {
    let song = read_gpx("test/tuplets2.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_test_irr_tuplet() {
    let song = read_gpx("test/testIrrTuplet.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_repeats() {
    let song = read_gpx("test/repeats.gpx");
    assert!(!song.measure_headers.is_empty());
    let has_repeat = song
        .measure_headers
        .iter()
        .any(|mh| mh.repeat_open || mh.repeat_close > 0);
    assert!(
        has_repeat,
        "repeats.gpx should have at least one repeat marker"
    );
}
#[test]
fn test_gpx_repeated_bars() {
    let song = read_gpx("test/repeated-bars.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_volta() {
    let song = read_gpx("test/volta.gpx");
    assert!(!song.measure_headers.is_empty());
    let has_volta = song
        .measure_headers
        .iter()
        .any(|mh| mh.repeat_alternative > 0);
    assert!(
        has_volta,
        "volta.gpx should have at least one alternate ending"
    );
}
#[test]
fn test_gpx_multivoices() {
    let song = read_gpx("test/multivoices.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_double_bar() {
    let song = read_gpx("test/double-bar.gpx");
    assert!(!song.measure_headers.is_empty());
    let has_double_bar = song.measure_headers.iter().any(|mh| mh.double_bar);
    assert!(
        has_double_bar,
        "double-bar.gpx should have at least one double bar"
    );
}
#[test]
fn test_gpx_clefs() {
    let song = read_gpx("test/clefs.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_bend() {
    let song = read_gpx("test/bend.gpx");
    assert!(!song.tracks.is_empty());
    // Verify that at least one note has a bend effect
    let has_bend = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.bend.is_some()))
            })
        })
    });
    assert!(
        has_bend,
        "bend.gpx should contain at least one note with a bend effect"
    );
}
#[test]
fn test_gpx_basic_bend() {
    let song = read_gpx("test/basic-bend.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_vibrato() {
    let song = read_gpx("test/vibrato.gpx");
    assert!(!song.tracks.is_empty());
    let has_vibrato = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.vibrato))
            })
        })
    });
    assert!(
        has_vibrato,
        "vibrato.gpx should contain at least one note with vibrato"
    );
}
#[test]
fn test_gpx_let_ring() {
    let song = read_gpx("test/let-ring.gpx");
    assert!(!song.tracks.is_empty());
    let has_let_ring = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.let_ring))
            })
        })
    });
    assert!(
        has_let_ring,
        "let-ring.gpx should contain at least one let-ring note"
    );
}
#[test]
fn test_gpx_palm_mute() {
    let song = read_gpx("test/palm-mute.gpx");
    assert!(!song.tracks.is_empty());
    let has_palm_mute = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.palm_mute))
            })
        })
    });
    assert!(
        has_palm_mute,
        "palm-mute.gpx should contain at least one palm-muted note"
    );
}
#[test]
fn test_gpx_accent() {
    let song = read_gpx("test/accent.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_sforzato() {
    let song = read_gpx("test/sforzato.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_heavy_accent() {
    let song = read_gpx("test/heavy-accent.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_ghost_note() {
    let song = read_gpx("test/ghost-note.gpx");
    assert!(!song.tracks.is_empty());
    let has_ghost = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.ghost_note))
            })
        })
    });
    assert!(
        has_ghost,
        "ghost-note.gpx should contain at least one ghost note"
    );
}
#[test]
fn test_gpx_dead_note() {
    use crate::model::enums::NoteType;
    let song = read_gpx("test/dead-note.gpx");
    assert!(!song.tracks.is_empty());
    let has_dead = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.kind == NoteType::Dead))
            })
        })
    });
    assert!(
        has_dead,
        "dead-note.gpx should contain at least one dead note"
    );
}
#[test]
fn test_gpx_trill() {
    let song = read_gpx("test/trill.gpx");
    assert!(!song.tracks.is_empty());
    let has_trill = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.trill.is_some()))
            })
        })
    });
    assert!(
        has_trill,
        "trill.gpx should contain at least one trill note"
    );
}
#[test]
fn test_gpx_tremolos() {
    let song = read_gpx("test/tremolos.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_grace() {
    let song = read_gpx("test/grace.gpx");
    assert!(!song.tracks.is_empty());
    let has_grace = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.grace.is_some()))
            })
        })
    });
    assert!(
        has_grace,
        "grace.gpx should contain at least one grace note"
    );
}
#[test]
fn test_gpx_grace_before_beat() {
    let song = read_gpx("test/grace-before-beat.gpx");
    assert!(!song.tracks.is_empty());
    let has_grace_before = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats.iter().any(|b| {
                    b.notes
                        .iter()
                        .any(|n| n.effect.grace.as_ref().is_some_and(|g| !g.is_on_beat))
                })
            })
        })
    });
    assert!(
        has_grace_before,
        "grace-before-beat.gpx should contain a grace note before the beat"
    );
}
#[test]
fn test_gpx_grace_on_beat() {
    let song = read_gpx("test/grace-on-beat.gpx");
    assert!(!song.tracks.is_empty());
    let has_grace_on = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats.iter().any(|b| {
                    b.notes
                        .iter()
                        .any(|n| n.effect.grace.as_ref().is_some_and(|g| g.is_on_beat))
                })
            })
        })
    });
    assert!(
        has_grace_on,
        "grace-on-beat.gpx should contain a grace note on the beat"
    );
}
#[test]
fn test_gpx_artificial_harmonic() {
    let song = read_gpx("test/artificial-harmonic.gpx");
    assert!(!song.tracks.is_empty());
    let has_harmonic = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.harmonic.is_some()))
            })
        })
    });
    assert!(
        has_harmonic,
        "artificial-harmonic.gpx should contain at least one harmonic note"
    );
}
#[test]
fn test_gpx_high_pitch() {
    let song = read_gpx("test/high-pitch.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_shift_slide() {
    let song = read_gpx("test/shift-slide.gpx");
    assert!(!song.tracks.is_empty());
    let has_slide = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| !n.effect.slides.is_empty()))
            })
        })
    });
    assert!(
        has_slide,
        "shift-slide.gpx should contain at least one note with slide effect"
    );
}
#[test]
fn test_gpx_legato_slide() {
    let song = read_gpx("test/legato-slide.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_slide_out_down() {
    let song = read_gpx("test/slide-out-down.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_slide_out_up() {
    let song = read_gpx("test/slide-out-up.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_slide_in_below() {
    let song = read_gpx("test/slide-in-below.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_slide_in_above() {
    let song = read_gpx("test/slide-in-above.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_brush() {
    let song = read_gpx("test/brush.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_arpeggio() {
    let song = read_gpx("test/arpeggio.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_rasg() {
    let song = read_gpx("test/rasg.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_fade_in() {
    let song = read_gpx("test/fade-in.gpx");
    assert!(!song.tracks.is_empty());
    let has_fade_in = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices
                .iter()
                .any(|v| v.beats.iter().any(|b| b.effect.fade_in))
        })
    });
    assert!(
        has_fade_in,
        "fade-in.gpx should contain at least one beat with fade-in"
    );
}
#[test]
fn test_gpx_volume_swell() {
    let song = read_gpx("test/volume-swell.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_pick_up_down() {
    let song = read_gpx("test/pick-up-down.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_slur() {
    let song = read_gpx("test/slur.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_slur_hammer_slur() {
    let song = read_gpx("test/slur_hammer_slur.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_slur_slur_hammer() {
    let song = read_gpx("test/slur_slur_hammer.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_slur_over_3_measures() {
    let song = read_gpx("test/slur_over_3_measures.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_slur_voices() {
    let song = read_gpx("test/slur_voices.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_slur_notes_effect_mask() {
    let song = read_gpx("test/slur-notes-effect-mask.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_dotted_gliss() {
    let song = read_gpx("test/dotted-gliss.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_ottava1() {
    let song = read_gpx("test/ottava1.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_ottava2() {
    let song = read_gpx("test/ottava2.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_ottava3() {
    let song = read_gpx("test/ottava3.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_ottava4() {
    let song = read_gpx("test/ottava4.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_ottava5() {
    let song = read_gpx("test/ottava5.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_mordents() {
    let song = read_gpx("test/mordents.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_turn() {
    let song = read_gpx("test/turn.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_barre() {
    let song = read_gpx("test/barre.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_fingering() {
    let song = read_gpx("test/fingering.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_fret_diagram() {
    let song = read_gpx("test/fret-diagram.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_fret_diagram_2instruments() {
    let song = read_gpx("test/fret-diagram_2instruments.gpx");
    assert!(song.tracks.len() >= 2);
}
#[test]
fn test_gpx_text() {
    let song = read_gpx("test/text.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_timer() {
    let song = read_gpx("test/timer.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_directions() {
    let song = read_gpx("test/directions.gpx");
    assert!(!song.measure_headers.is_empty());
}
#[test]
fn test_gpx_fermata() {
    let song = read_gpx("test/fermata.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_free_time() {
    let song = read_gpx("test/free-time.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_dynamic() {
    let song = read_gpx("test/dynamic.gpx");
    assert!(!song.tracks.is_empty());
    // Verify that notes have varying velocities (not all the same default)
    let velocities: Vec<i16> = song
        .tracks
        .iter()
        .flat_map(|t| {
            t.measures.iter().flat_map(|m| {
                m.voices.iter().flat_map(|v| {
                    v.beats
                        .iter()
                        .flat_map(|b| b.notes.iter().map(|n| n.velocity))
                })
            })
        })
        .collect();
    assert!(!velocities.is_empty(), "dynamic.gpx should contain notes");
    let has_varying = velocities.iter().any(|&v| v != velocities[0]);
    assert!(
        has_varying,
        "dynamic.gpx should have varying velocities across notes"
    );
}
#[test]
fn test_gpx_crescendo_diminuendo() {
    let song = read_gpx("test/crescendo-diminuendo.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_wah() {
    let song = read_gpx("test/wah.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_all_percussion() {
    let song = read_gpx("test/all-percussion.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_beams_stems_ledger_lines() {
    let song = read_gpx("test/beams-stems-ledger-lines.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_chordnames_keyboard() {
    let song = read_gpx("test/chordnames_keyboard.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_tuplet_with_slur() {
    let song = read_gpx("test/tuplet-with-slur.gpx");
    assert!(!song.tracks.is_empty());
}

#[test]
fn test_gpx_all_files_parse() {
    use std::fs;
    let test_dir = "../test";
    let mut pass = 0;
    let mut failures: Vec<String> = Vec::new();
    for entry in fs::read_dir(test_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "gpx") {
            let fname = path.file_name().unwrap().to_str().unwrap().to_string();
            let data = fs::read(&path).unwrap();
            let mut song = Song::default();
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                song.read_gpx(&data).unwrap();
            })) {
                Ok(_) => {
                    pass += 1;
                }
                Err(e) => {
                    let msg = if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else {
                        "unknown".to_string()
                    };
                    let short = &msg[..msg.len().min(100)];
                    failures.push(format!("{}: {}", fname, short));
                }
            }
        }
    }
    if !failures.is_empty() {
        for f in &failures {
            eprintln!("FAIL: {}", f);
        }
    }
    eprintln!(
        "{} pass, {} fail out of {}",
        pass,
        failures.len(),
        pass + failures.len()
    );
    assert!(
        failures.is_empty(),
        "{} files failed to parse",
        failures.len()
    );
}

// ==================== GP7 (Guitar Pro 7+) tests ====================

fn read_gp7(filename: &str) -> Song {
    let mut song = Song::default();
    song.read_gp(&read_file(String::from(filename))).unwrap();
    song
}

#[test]
fn test_gp7_keysig() {
    let song = read_gp7("test/keysig.gp");
    assert_eq!(song.tracks.len(), 1);
    assert_eq!(song.measure_headers.len(), 32);
}
#[test]
fn test_gp7_copyright() {
    let song = read_gp7("test/copyright.gp");
    assert!(!song.tracks.is_empty());
    assert!(
        !song.copyright.is_empty(),
        "copyright field should be populated"
    );
}
#[test]
fn test_gp7_tempo() {
    let song = read_gp7("test/tempo.gp");
    assert!(!song.measure_headers.is_empty());
    assert!(song.tempo > 0, "tempo should be parsed from automations");
}
#[test]
fn test_gp7_rest_centered() {
    let song = read_gp7("test/rest-centered.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_dotted_tuplets() {
    let song = read_gp7("test/dotted-tuplets.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_test_irr_tuplet() {
    let song = read_gp7("test/testIrrTuplet.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_repeats() {
    let song = read_gp7("test/repeats.gp");
    assert!(!song.measure_headers.is_empty());
    let has_repeat = song
        .measure_headers
        .iter()
        .any(|mh| mh.repeat_open || mh.repeat_close > 0);
    assert!(
        has_repeat,
        "repeats.gp should have at least one repeat marker"
    );
}
#[test]
fn test_gp7_repeated_bars() {
    let song = read_gp7("test/repeated-bars.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_volta() {
    let song = read_gp7("test/volta.gp");
    assert!(!song.measure_headers.is_empty());
    let has_volta = song
        .measure_headers
        .iter()
        .any(|mh| mh.repeat_alternative > 0);
    assert!(
        has_volta,
        "volta.gp should have at least one alternate ending"
    );
}
#[test]
fn test_gp7_multivoices() {
    let song = read_gp7("test/multivoices.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_double_bar() {
    let song = read_gp7("test/double-bar.gp");
    assert!(!song.measure_headers.is_empty());
    let has_double_bar = song.measure_headers.iter().any(|mh| mh.double_bar);
    assert!(
        has_double_bar,
        "double-bar.gp should have at least one double bar"
    );
}
#[test]
fn test_gp7_clefs() {
    let song = read_gp7("test/clefs.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_bend() {
    let song = read_gp7("test/bend.gp");
    assert!(!song.tracks.is_empty());
    let has_bend = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.bend.is_some()))
            })
        })
    });
    assert!(
        has_bend,
        "bend.gp should contain at least one note with a bend effect"
    );
}
#[test]
fn test_gp7_basic_bend() {
    let song = read_gp7("test/basic-bend.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_vibrato() {
    let song = read_gp7("test/vibrato.gp");
    assert!(!song.tracks.is_empty());
    let has_vibrato = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.vibrato))
            })
        })
    });
    assert!(
        has_vibrato,
        "vibrato.gp should contain at least one note with vibrato"
    );
}
#[test]
fn test_gp7_let_ring() {
    let song = read_gp7("test/let-ring.gp");
    assert!(!song.tracks.is_empty());
    let has_let_ring = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.let_ring))
            })
        })
    });
    assert!(
        has_let_ring,
        "let-ring.gp should contain at least one let-ring note"
    );
}
#[test]
fn test_gp7_palm_mute() {
    let song = read_gp7("test/palm-mute.gp");
    assert!(!song.tracks.is_empty());
    let has_palm_mute = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.palm_mute))
            })
        })
    });
    assert!(
        has_palm_mute,
        "palm-mute.gp should contain at least one palm-muted note"
    );
}
#[test]
fn test_gp7_accent() {
    let song = read_gp7("test/accent.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_sforzato() {
    let song = read_gp7("test/sforzato.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_heavy_accent() {
    let song = read_gp7("test/heavy-accent.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_ghost_note() {
    let song = read_gp7("test/ghost-note.gp");
    assert!(!song.tracks.is_empty());
    let has_ghost = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.ghost_note))
            })
        })
    });
    assert!(
        has_ghost,
        "ghost-note.gp should contain at least one ghost note"
    );
}
#[test]
fn test_gp7_dead_note() {
    use crate::model::enums::NoteType;
    let song = read_gp7("test/dead-note.gp");
    assert!(!song.tracks.is_empty());
    let has_dead = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.kind == NoteType::Dead))
            })
        })
    });
    assert!(
        has_dead,
        "dead-note.gp should contain at least one dead note"
    );
}
#[test]
fn test_gp7_trill() {
    let song = read_gp7("test/trill.gp");
    assert!(!song.tracks.is_empty());
    let has_trill = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.trill.is_some()))
            })
        })
    });
    assert!(has_trill, "trill.gp should contain at least one trill note");
}
#[test]
fn test_gp7_tremolos() {
    let song = read_gp7("test/tremolos.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_grace() {
    let song = read_gp7("test/grace.gp");
    assert!(!song.tracks.is_empty());
    let has_grace = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.grace.is_some()))
            })
        })
    });
    assert!(has_grace, "grace.gp should contain at least one grace note");
}
#[test]
fn test_gp7_grace_before_beat() {
    let song = read_gp7("test/grace-before-beat.gp");
    assert!(!song.tracks.is_empty());
    let has_grace_before = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats.iter().any(|b| {
                    b.notes
                        .iter()
                        .any(|n| n.effect.grace.as_ref().is_some_and(|g| !g.is_on_beat))
                })
            })
        })
    });
    assert!(
        has_grace_before,
        "grace-before-beat.gp should contain a grace note before the beat"
    );
}
#[test]
fn test_gp7_grace_on_beat() {
    let song = read_gp7("test/grace-on-beat.gp");
    assert!(!song.tracks.is_empty());
    let has_grace_on = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats.iter().any(|b| {
                    b.notes
                        .iter()
                        .any(|n| n.effect.grace.as_ref().is_some_and(|g| g.is_on_beat))
                })
            })
        })
    });
    assert!(
        has_grace_on,
        "grace-on-beat.gp should contain a grace note on the beat"
    );
}
#[test]
fn test_gp7_artificial_harmonic() {
    let song = read_gp7("test/artificial-harmonic.gp");
    assert!(!song.tracks.is_empty());
    let has_harmonic = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.harmonic.is_some()))
            })
        })
    });
    assert!(
        has_harmonic,
        "artificial-harmonic.gp should contain at least one harmonic note"
    );
}
#[test]
fn test_gp7_high_pitch() {
    let song = read_gp7("test/high-pitch.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_shift_slide() {
    let song = read_gp7("test/shift-slide.gp");
    assert!(!song.tracks.is_empty());
    let has_slide = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| !n.effect.slides.is_empty()))
            })
        })
    });
    assert!(
        has_slide,
        "shift-slide.gp should contain at least one note with slide effect"
    );
}
#[test]
fn test_gp7_legato_slide() {
    let song = read_gp7("test/legato-slide.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_slide_out_down() {
    let song = read_gp7("test/slide-out-down.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_slide_out_up() {
    let song = read_gp7("test/slide-out-up.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_slide_in_below() {
    let song = read_gp7("test/slide-in-below.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_slide_in_above() {
    let song = read_gp7("test/slide-in-above.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_brush() {
    let song = read_gp7("test/brush.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_arpeggio() {
    let song = read_gp7("test/arpeggio.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_rasg() {
    let song = read_gp7("test/rasg.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_fade_in() {
    let song = read_gp7("test/fade-in.gp");
    assert!(!song.tracks.is_empty());
    let has_fade_in = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices
                .iter()
                .any(|v| v.beats.iter().any(|b| b.effect.fade_in))
        })
    });
    assert!(
        has_fade_in,
        "fade-in.gp should contain at least one beat with fade-in"
    );
}
#[test]
fn test_gp7_volume_swell() {
    let song = read_gp7("test/volume-swell.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_pick_up_down() {
    let song = read_gp7("test/pick-up-down.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_slur() {
    let song = read_gp7("test/slur.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_slur_hammer_slur() {
    let song = read_gp7("test/slur_hammer_slur.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_slur_slur_hammer() {
    let song = read_gp7("test/slur_slur_hammer.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_slur_over_3_measures() {
    let song = read_gp7("test/slur_over_3_measures.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_slur_voices() {
    let song = read_gp7("test/slur_voices.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_slur_notes_effect_mask() {
    let song = read_gp7("test/slur-notes-effect-mask.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_dotted_gliss() {
    let song = read_gp7("test/dotted-gliss.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_ottava1() {
    let song = read_gp7("test/ottava1.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_ottava2() {
    let song = read_gp7("test/ottava2.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_ottava3() {
    let song = read_gp7("test/ottava3.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_ottava4() {
    let song = read_gp7("test/ottava4.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_ottava5() {
    let song = read_gp7("test/ottava5.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_mordents() {
    let song = read_gp7("test/mordents.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_turn() {
    let song = read_gp7("test/turn.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_barre() {
    let song = read_gp7("test/barre.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_fingering() {
    let song = read_gp7("test/fingering.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_fret_diagram() {
    let song = read_gp7("test/fret-diagram.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_fret_diagram_2instruments() {
    let song = read_gp7("test/fret-diagram_2instruments.gp");
    assert!(song.tracks.len() >= 2);
}
#[test]
fn test_gp7_text() {
    let song = read_gp7("test/text.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_timer() {
    let song = read_gp7("test/timer.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_directions() {
    let song = read_gp7("test/directions.gp");
    assert!(!song.measure_headers.is_empty());
}
#[test]
fn test_gp7_fermata() {
    let song = read_gp7("test/fermata.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_free_time() {
    let song = read_gp7("test/free-time.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_dynamic() {
    let song = read_gp7("test/dynamic.gp");
    assert!(!song.tracks.is_empty());
    let velocities: Vec<i16> = song
        .tracks
        .iter()
        .flat_map(|t| {
            t.measures.iter().flat_map(|m| {
                m.voices.iter().flat_map(|v| {
                    v.beats
                        .iter()
                        .flat_map(|b| b.notes.iter().map(|n| n.velocity))
                })
            })
        })
        .collect();
    assert!(!velocities.is_empty(), "dynamic.gp should contain notes");
    let has_varying = velocities.iter().any(|&v| v != velocities[0]);
    assert!(
        has_varying,
        "dynamic.gp should have varying velocities across notes"
    );
}
#[test]
fn test_gp7_crescendo_diminuendo() {
    let song = read_gp7("test/crescendo-diminuendo.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_wah() {
    let song = read_gp7("test/wah.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_all_percussion() {
    let song = read_gp7("test/all-percussion.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_beams_stems_ledger_lines() {
    let song = read_gp7("test/beams-stems-ledger-lines.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_chordnames_keyboard() {
    let song = read_gp7("test/chordnames_keyboard.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_tuplet_with_slur() {
    let song = read_gp7("test/tuplet-with-slur.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_tap_slap_pop() {
    let song = read_gp7("test/tap-slap-pop.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_tremolo_bar() {
    let song = read_gp7("test/tremolo-bar.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_test() {
    let song = read_gp7("test/test.gp");
    assert!(!song.tracks.is_empty());
}

#[test]
fn test_gp7_all_files_parse() {
    use std::fs;
    let test_dir = "../test";
    let mut pass = 0;
    let mut failures: Vec<String> = Vec::new();
    for entry in fs::read_dir(test_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "gp") {
            let fname = path.file_name().unwrap().to_str().unwrap().to_string();
            let data = fs::read(&path).unwrap();
            let mut song = Song::default();
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                song.read_gp(&data).unwrap();
            })) {
                Ok(_) => {
                    pass += 1;
                }
                Err(e) => {
                    let msg = if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else {
                        "unknown".to_string()
                    };
                    let short = &msg[..msg.len().min(100)];
                    failures.push(format!("{}: {}", fname, short));
                }
            }
        }
    }
    if !failures.is_empty() {
        for f in &failures {
            eprintln!("FAIL: {}", f);
        }
    }
    eprintln!(
        "{} pass, {} fail out of {}",
        pass,
        failures.len(),
        pass + failures.len()
    );
    assert!(
        failures.is_empty(),
        "{} files failed to parse",
        failures.len()
    );
}
