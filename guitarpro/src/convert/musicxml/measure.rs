//! Per-measure conversion: attributes, barlines, tempo direction, voices.

use crate::{
    convert::musicxml::{DIVISIONS, helpers, note},
    model::{
        legacy::{headers::MeasureHeader, measure::Measure, song::Song, track::Track},
        musicxml::{self, attributes::Clef},
    },
};

// ---------------------------------------------------------------------------
// Clef helper
// ---------------------------------------------------------------------------

pub(super) fn make_clef(sign: &str, line: Option<u8>, number: Option<u8>) -> Clef {
    Clef {
        number,
        additional: None,
        size: None,
        after_barline: None,
        print_object: None,
        sign: sign.to_string(),
        line,
        clef_octave_change: None,
    }
}

// ---------------------------------------------------------------------------
// Measure
// ---------------------------------------------------------------------------

pub(super) fn build_measure(
    song: &Song,
    track: &Track,
    track_idx: usize,
    measure: &Measure,
    header: &MeasureHeader,
    measure_idx: usize,
) -> musicxml::measure::Measure {
    use musicxml::{
        attributes::{Attributes, Key, StaffDetails, StaffTuning, Time},
        barline::Barline,
        direction::{Direction, DirectionType, DirectionTypeWrapper, Sound},
        measure::MusicData,
    };

    let mut music_data: Vec<MusicData> = vec![];

    // --- Attributes (first measure only) ---
    if measure_idx == 0 {
        let notation_clef = match measure.clef {
            crate::model::legacy::enums::MeasureClef::Treble => make_clef("G", Some(2), None),
            crate::model::legacy::enums::MeasureClef::Bass => make_clef("F", Some(4), None),
            crate::model::legacy::enums::MeasureClef::Tenor => make_clef("C", Some(4), None),
            crate::model::legacy::enums::MeasureClef::Alto => make_clef("C", Some(3), None),
        };

        // GP strings: high→low (string 1 = highest). MusicXML line 1 = lowest.
        let string_count = track.strings.len();
        let staff_tunings: Vec<StaffTuning> = track
            .strings
            .iter()
            .rev()
            .enumerate()
            .map(|(i, (_str_num, midi_val))| {
                let (step, alter, octave) = helpers::midi_to_pitch(*midi_val);
                StaffTuning {
                    line: (i + 1) as u8,
                    tuning_step: step,
                    tuning_alter: alter,
                    tuning_octave: octave,
                }
            })
            .collect();

        music_data.push(MusicData::Attributes(Attributes {
            divisions: Some(DIVISIONS),
            keys: vec![Key {
                number: None,
                print_object: None,
                cancel: None,
                fifths: Some(header.key_signature.key),
                mode: Some(
                    if header.key_signature.is_minor {
                        "minor"
                    } else {
                        "major"
                    }
                    .to_string(),
                ),
                key_steps: vec![],
                key_alters: vec![],
                key_accidentals: vec![],
                key_octaves: vec![],
            }],
            times: vec![Time {
                number: None,
                symbol: None,
                separator: None,
                print_object: None,
                beats: Some(header.time_signature.numerator.to_string()),
                beat_type: Some(header.time_signature.denominator.value.to_string()),
                senza_misura: None,
                interchangeable: None,
            }],
            staves: Some(2),
            part_symbol: None,
            instruments: None,
            clefs: vec![notation_clef, make_clef("TAB", None, Some(2))],
            staff_details: vec![StaffDetails {
                number: Some(2),
                show_frets: None,
                print_object: None,
                print_spacing: None,
                staff_type: Some("tab".to_string()),
                staff_lines: Some(string_count as u8),
                line_details: vec![],
                staff_tunings,
                capo: None,
                staff_size: None,
            }],
            transposes: vec![],
            for_parts: vec![],
            directives: vec![],
            measure_styles: vec![],
        }));
    }

    // --- Tempo direction (first measure of first track only) ---
    if measure_idx == 0 && track_idx == 0 {
        let tempo_bpm = if header.tempo > 0 {
            header.tempo as f64
        } else {
            song.tempo as f64
        };
        music_data.push(MusicData::Direction(Direction {
            placement: Some("above".to_string()),
            directive: None,
            id: None,
            direction_types: vec![DirectionTypeWrapper {
                content: DirectionType::Metronome(musicxml::direction::Metronome {
                    parentheses: None,
                    default_x: None,
                    default_y: None,
                    justify: None,
                    id: None,
                    beat_unit: Some("quarter".to_string()),
                    beat_unit_dots: vec![],
                    beat_unit_tied: None,
                    per_minute: Some(musicxml::direction::PerMinute {
                        font_size: None,
                        value: format!("{}", tempo_bpm as u32),
                    }),
                    metronome_arrows: None,
                    metronome_notes: vec![],
                    metronome_relation: None,
                }),
            }],
            offset: None,
            footnote: None,
            level: None,
            voice: None,
            staff: None,
            sound: Some(Sound {
                tempo: Some(tempo_bpm),
                dynamics: None,
                dacapo: None,
                segno: None,
                dalsegno: None,
                coda: None,
                tocoda: None,
                divisions: None,
                forward_repeat: None,
                fine: None,
                time_only: None,
                pizzicato: None,
                pan: None,
                elevation: None,
                damper_pedal: None,
                soft_pedal: None,
                sostenuto_pedal: None,
                id: None,
                midi_devices: vec![],
                midi_instruments: vec![],
                plays: vec![],
                swing: None,
                offset: None,
            }),
            listening: None,
            wavy_line: None,
            dashes: None,
            bracket: None,
            pedal: None,
        }));
    }

    // --- Repeat open barline ---
    if header.repeat_open {
        music_data.insert(
            0,
            MusicData::Barline(Barline {
                location: Some("left".to_string()),
                segno: None,
                coda: None,
                divisions: None,
                id: None,
                bar_style: Some(musicxml::barline::BarStyleColor {
                    color: None,
                    value: musicxml::barline::BarStyle::HeavyLight,
                }),
                footnote: None,
                level: None,
                wavy_line: None,
                segno_mark: None,
                coda_mark: None,
                ending: None,
                repeat: Some(musicxml::barline::Repeat {
                    direction: "forward".to_string(),
                    times: None,
                    winged: None,
                }),
                fermatas: vec![],
            }),
        );
    }

    // --- Repeat close barline ---
    if header.repeat_close > 0 {
        music_data.push(MusicData::Barline(Barline {
            location: Some("right".to_string()),
            segno: None,
            coda: None,
            divisions: None,
            id: None,
            bar_style: Some(musicxml::barline::BarStyleColor {
                color: None,
                value: musicxml::barline::BarStyle::LightHeavy,
            }),
            footnote: None,
            level: None,
            wavy_line: None,
            segno_mark: None,
            coda_mark: None,
            ending: None,
            repeat: Some(musicxml::barline::Repeat {
                direction: "backward".to_string(),
                times: Some(header.repeat_close as u8),
                winged: None,
            }),
            fermatas: vec![],
        }));
    }

    // --- Voices → notes ---
    music_data.extend(note::build_voices(measure, track));

    musicxml::measure::Measure {
        number: header.number.to_string(),
        implicit: None,
        non_controlling: None,
        width: None,
        text: None,
        id: None,
        music_data,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_clef_treble() {
        let clef = make_clef("G", Some(2), None);
        assert_eq!(clef.sign, "G");
        assert_eq!(clef.line, Some(2));
        assert!(clef.number.is_none());
    }

    #[test]
    fn make_clef_bass() {
        let clef = make_clef("F", Some(4), None);
        assert_eq!(clef.sign, "F");
        assert_eq!(clef.line, Some(4));
    }

    #[test]
    fn make_clef_tab_has_staff_number() {
        let clef = make_clef("TAB", None, Some(2));
        assert_eq!(clef.sign, "TAB");
        assert!(clef.line.is_none());
        assert_eq!(clef.number, Some(2));
    }

    #[test]
    fn make_clef_fields_default_to_none() {
        let clef = make_clef("G", Some(2), None);
        assert!(clef.additional.is_none());
        assert!(clef.size.is_none());
        assert!(clef.after_barline.is_none());
        assert!(clef.print_object.is_none());
        assert!(clef.clef_octave_change.is_none());
    }
}
