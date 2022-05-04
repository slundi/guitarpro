#[path = "song.rs"] pub mod gp;
mod io;
pub mod enums;
pub mod headers;
pub mod track;
pub mod measure;
pub mod effects;
pub mod key_signature;
pub mod midi;
pub mod mix_table;
pub mod chord;
pub mod page;
pub mod rse;
pub mod note;
pub mod lyric;
pub mod beat;

#[cfg(test)]
mod test {
    use std::{io::Read, fs};
    use fraction::ToPrimitive;
    use crate::gp::Song;

    fn read_file(path: String) -> Vec<u8> {
        let f = fs::OpenOptions::new().read(true).open(&path).expect("Cannot open file");
        let size: usize = fs::metadata(&path).unwrap_or_else(|_e|{panic!("Unable to get file size")}).len().to_usize().unwrap();
        let mut data: Vec<u8> = Vec::with_capacity(size);
        f.take(u64::from_ne_bytes(size.to_ne_bytes())).read_to_end(&mut data).unwrap_or_else(|_error|{panic!("Unable to read file contents");});
        data
    }

    //chords
    #[test]
    fn test_gp3_chord() {
        let mut song: Song = Song::default();
        song.read_gp3(&read_file(String::from("test/Chords.gp3")));
    }
    #[test]
    fn test_gp4_chord() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/Chords.gp4")));
    }
    #[test]
    fn test_gp5_chord() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Chords.gp5")));
    }
    #[test]
    fn test_gp5_unknown_chord_extension() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Unknown Chord Extension.gp5")));
    }
    #[test]
    fn test_gp5_chord_without_notes() { //Read chord even if there's no fingering
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/chord_without_notes.gp5")));
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/001_Funky_Guy.gp5")));
    }

    //duration
    #[test]
    fn test_gp3_duration() {
        let mut song: Song = Song::default();
        song.read_gp3(&read_file(String::from("test/Duration.gp3")));
    }

    //effects
    #[test]
    fn test_gp3_effects() {
        let mut song: Song = Song::default();
        song.read_gp3(&read_file(String::from("test/Effects.gp3")));
    }
    #[test]
    fn test_gp4_effects() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/Effects.gp4")));
    }
    #[test]
    fn test_gp5_effects() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Effects.gp5")));
    }

    //harmonics
    #[test]
    fn test_gp3_harmonics() {
        let mut song: Song = Song::default();
        song.read_gp3(&read_file(String::from("test/Harmonics.gp3")));
    }
    #[test]
    fn test_gp4_harmonics() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/Harmonics.gp4")));
    }
    #[test]
    fn test_gp5_harmonics() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Harmonics.gp5")));
    }

    //key
    #[test]
    fn test_gp4_key() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/Key.gp4")));
    }
    #[test]
    fn test_gp5_key() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Key.gp5")));
    }

    //demo

    //repeat
    #[test]
    fn test_gp4_repeat() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/Repeat.gp4")));
    }
    #[test]
    fn test_gp5_repeat() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Repeat.gp5")));
    }

    //RSE
    #[test]
    fn test_gp5_rse() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/RSE.gp5")));
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Demo v5.gp5")));
    }

    //slides
    #[test]
    fn test_gp4_slides() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/Slides.gp4")));
    }
    #[test]
    fn test_gp5_slides() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Slides.gp5")));
    }

    //strokes
    #[test]
    fn test_gp4_strokes() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/Strokes.gp4")));
    }
    #[test]
    fn test_gp5_strokes() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Strokes.gp5")));
    }

    //vibrato
    #[test]
    fn test_gp4_vibrato() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/Vibrato.gp4")));
    }

    //voices
    #[test]
    fn test_gp5_voices() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Voices.gp5")));
    }

    //wah
    #[test]
    fn test_gp5_no_wah() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/No Wah.gp5")));
    }
    #[test]
    fn test_gp5_wah() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Wah.gp5")));
    }
    #[test]
    fn test_gp5_wah_m() { //Handle gradual wah-wah changes
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/Wah-m.gp5")));
    }

    //MuseScore tests
    #[test]
    fn test_gp5_all_percussion() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/all-percussion.gp5")));
    }
    #[test]
    fn test_gp5_basic_bend() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/basic-bend.gp5")));
    }
    #[test]
    fn test_gp5_beams_sterms_ledger_lines() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/beams-stems-ledger-lines.gp5")));
    }
    #[test]
    fn test_gp5_brush() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/brush.gp5")));
    }
    #[test]
    fn test_gp3_capo_fret() {
        let mut song: Song = Song::default();
        song.read_gp3(&read_file(String::from("test/capo-fret.gp3")));
    }
    #[test]
    fn test_gp4_capo_fret() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/capo-fret.gp4")));
    }
    #[test]
    fn test_gp5_capo_fret() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/capo-fret.gp5")));
    }
    #[test]
    fn test_gp3_copyright() {
        let mut song: Song = Song::default();
        song.read_gp3(&read_file(String::from("test/copyright.gp3")));
    }
    #[test]
    fn test_gp4_copyright() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/copyright.gp4")));
    }
    #[test]
    fn test_gp5_copyright() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/copyright.gp5")));
    }
    #[test]
    fn test_gp3_dotted_gliss() {
        let mut song: Song = Song::default();
        song.read_gp3(&read_file(String::from("test/dotted-gliss.gp3")));
    }
    #[test]
    fn test_gp5_dotted_tuplets() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/dotted-tuplets.gp5")));
    }
    #[test]
    fn test_gp5_dynamic() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/dynamic.gp5")));
    }
    #[test]
    fn test_gp4_fade_in() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/fade-in.gp4")));
    }
    #[test]
    fn test_gp5_fade_in() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/fade-in.gp5")));
    }
    #[test]
    fn test_gp4_fingering() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/fingering.gp4")));
    }
    #[test]
    fn test_gp5_fingering() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/fingering.gp5")));
    }
    #[test]
    fn test_gp4_fret_diagram() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/fret-diagram.gp4")));
    }
    #[test]
    fn test_gp5_fret_diagram() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/fret-diagram.gp5")));
    }
    #[test]
    fn test_gp3_ghost_note() {
        let mut song: Song = Song::default();
        song.read_gp3(&read_file(String::from("test/ghost_note.gp3")));
    }
    #[test]
    fn test_gp5_grace() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/grace.gp5")));
    }
    #[test]
    fn test_gp5_heavy_accent() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/heavy-accent.gp5")));
    }
    #[test]
    fn test_gp3_high_pitch() {
        let mut song: Song = Song::default();
        song.read_gp3(&read_file(String::from("test/high-pitch.gp3")));
    }
    #[test]
    fn test_gp4_keysig() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/keysig.gp4")));
    }
    #[test]
    fn test_gp5_keysig() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/keysig.gp5")));
    }
    #[test]
    fn test_gp4_legato_slide() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/legato-slide.gp4")));
    }
    #[test]
    fn test_gp5_legato_slide() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/legato-slide.gp5")));
    }
    #[test]
    fn test_gp4_let_ring() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/let-ring.gp4")));
    }
    #[test]
    fn test_gp5_let_ring() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/let-ring.gp5")));
    }
    #[test]
    fn test_gp4_palm_mute() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/palm-mute.gp4")));
    }
    #[test]
    fn test_gp5_palm_mute() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/palm-mute.gp5")));
    }
    #[test]
    fn test_gp4_pick_up_down() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/pick-up-down.gp4")));
    }
    #[test]
    fn test_gp5_pick_up_down() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/pick-up-down.gp5")));
    }
    #[test]
    fn test_gp4_rest_centered() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/rest-centered.gp4")));
    }
    #[test]
    fn test_gp5_rest_centered() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/rest-centered.gp5")));
    }
    #[test]
    fn test_gp4_sforzato() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/sforzato.gp4")));
    }
    #[test]
    fn test_gp4_shift_slide() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/shift-slide.gp4")));
    }
    #[test]
    fn test_gp5_shift_slide() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/shift-slide.gp5")));
    }
    #[test]
    fn test_gp4_slide_in_above() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/slide-in-above.gp4")));
    }
    #[test]
    fn test_gp5_slide_in_above() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/slide-in-above.gp5")));
    }
    #[test]
    fn test_gp4_slide_in_below() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/slide-in-below.gp4")));
    }
    #[test]
    fn test_gp5_slide_in_below() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/slide-in-below.gp5")));
    }
    #[test]
    fn test_gp4_slide_out_down() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/slide-out-down.gp4")));
    }
    #[test]
    fn test_gp5_slide_out_down() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/slide-out-down.gp5")));
    }
    #[test]
    fn test_gp4_slide_out_up() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/slide-out-up.gp4")));
    }
    #[test]
    fn test_gp5_slide_out_up() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/slide-out-up.gp5")));
    }
    #[test]
    fn test_gp4_slur() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/slur.gp4")));
    }
    #[test]
    fn test_gp5_slur_notes_effect_mask() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/slur-notes-effect-mask.gp5")));
    }
    #[test]
    fn test_gp5_tap_slap_pop() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/tap-slap-pop.gp5")));
    }
    #[test]
    fn test_gp3_tempo() {
        let mut song: Song = Song::default();
        song.read_gp3(&read_file(String::from("test/tempo.gp3")));
    }
    #[test]
    fn test_gp4_tempo() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/tempo.gp4")));
    }
    #[test]
    fn test_gp5_tempo() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/tempo.gp5")));
    }
    #[test]
    fn test_gp4_test_irr_tuplet() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/testIrrTuplet.gp4")));
    }
    #[test]
    fn test_gp5_tremolos() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/tremolos.gp5")));
    }
    #[test]
    fn test_gp4_trill() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/trill.gp4")));
    }
    #[test]
    fn test_gp4_tuplet_with_slur() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/tuplet-with-slur.gp4")));
    }
    #[test]
    fn test_gp5_vibrato() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/vibrato.gp5")));
    }
    #[test]
    fn test_gp3_volta() {
        let mut song: Song = Song::default();
        song.read_gp3(&read_file(String::from("test/volta.gp3")));
    }
    #[test]
    fn test_gp4_volta() {
        let mut song: Song = Song::default();
        song.read_gp4(&read_file(String::from("test/volta.gp4")));
    }
    #[test]
    fn test_gp5_volta() {
        let mut song: Song = Song::default();
        song.read_gp5(&read_file(String::from("test/volta.gp5")));
    }

    //writing
    #[test]
    fn test_gp3_writing() {
        let mut song = Song::default();
        let data = read_file(String::from("test/Chords.gp3"));
        song.read_gp3(&data);
        let out = song.write((3,0,0), None);
        assert_eq!(out, data[0..out.len()]);
        song.read_gp3(&out);
    }
}
