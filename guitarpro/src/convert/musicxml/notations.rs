//! `<notations>` builder: technical markings, ornaments, articulations, slides.

use crate::model::legacy::note::Note;
use crate::model::musicxml;

/// Build `<notations>` from a legacy [`Note`], including tablature fret/string.
pub(super) fn build_notations(note: &Note, strings: &[(i8, i8)]) -> Vec<musicxml::note::Notations> {
    use crate::model::legacy::enums::SlideType;
    use musicxml::note::{
        Articulations, Bend, Fret, HammerPull, Harmonic, Notations, Ornaments, PlacedEmpty, Slide,
        StringNumber, Technical, WavyLine,
    };

    let eff = &note.effect;
    let mut technical = Technical {
        up_bow: None,
        down_bow: None,
        harmonic: None,
        open_string: None,
        thumb_position: None,
        fingering: None,
        pluck: None,
        double_tongue: None,
        triple_tongue: None,
        stopped: None,
        snap_pizzicato: None,
        fret: None,
        string: None,
        hammer_on: None,
        pull_off: None,
        bend: None,
        tap: None,
        heel: None,
        toe: None,
        fingernails: None,
        hole: None,
        arrow: None,
        handbell: None,
        brass_bend: None,
        flip: None,
        smear: None,
        open: None,
        half_muted: None,
        harmon_mute: None,
        golpe: None,
        other_technical: None,
    };

    // Fret + string number for tablature
    if note.string > 0 && (note.string as usize) <= strings.len() {
        technical.fret = Some(Fret {
            font_size: None,
            color: None,
            value: note.value as u8,
        });
        technical.string = Some(StringNumber {
            default_x: None,
            default_y: None,
            placement: None,
            value: note.string as u8,
        });
    }

    // Hammer-on
    if eff.hammer {
        technical.hammer_on = Some(HammerPull {
            technique_type: "start".to_string(),
            number: None,
            placement: None,
            value: None,
        });
    }

    // Bend — GP bend value is in semitone quarters (100 units = 1 semitone).
    // Use the peak value (max of points).
    if let Some(bend) = &eff.bend {
        let peak = bend.points.iter().map(|p| p.value).max().unwrap_or(0);
        if peak > 0 {
            technical.bend = Some(Bend {
                shape: None,
                default_x: None,
                default_y: None,
                bend_alter: peak as f64 / 100.0,
                pre_bend: None,
                release: None,
                with_bar: None,
            });
        }
    }

    // Let ring — no direct MusicXML equivalent, encode as other-technical
    if eff.let_ring {
        technical.other_technical = Some(musicxml::note::OtherPlacement {
            placement: None,
            smufl: None,
            value: Some("let-ring".to_string()),
        });
    }

    // Palm mute → half-muted
    if eff.palm_mute {
        technical.half_muted = Some(musicxml::note::OtherPlacement {
            placement: None,
            smufl: None,
            value: None,
        });
    }

    // Natural harmonic
    if eff.harmonic.is_some() {
        technical.harmonic = Some(Harmonic {
            print_object: None,
            placement: None,
            natural: Some(()),
            artificial: None,
            base_pitch: None,
            touching_pitch: None,
            sounding_pitch: None,
        });
    }

    let has_technical = technical.fret.is_some()
        || technical.string.is_some()
        || technical.hammer_on.is_some()
        || technical.pull_off.is_some()
        || technical.bend.is_some()
        || technical.other_technical.is_some()
        || technical.harmonic.is_some()
        || technical.half_muted.is_some();
    let technical_opt = if has_technical { Some(technical) } else { None };

    // Slides → <slide> elements
    let slides: Vec<Slide> = eff
        .slides
        .iter()
        .filter_map(|s| match s {
            SlideType::ShiftSlideTo | SlideType::LegatoSlideTo => Some(Slide {
                slide_type: "start".to_string(),
                number: None,
                line_type: Some(
                    if *s == SlideType::LegatoSlideTo {
                        "solid"
                    } else {
                        "dashed"
                    }
                    .to_string(),
                ),
                value: None,
            }),
            _ => None,
        })
        .collect();

    // Vibrato → wavy-line ornament
    let ornaments = if eff.vibrato {
        Some(Ornaments {
            trill_mark: None,
            turn: None,
            delayed_turn: None,
            inverted_turn: None,
            delayed_inverted_turn: None,
            vertical_turn: None,
            inverted_vertical_turn: None,
            shake: None,
            wavy_line: Some(WavyLine {
                wavy_type: "start".to_string(),
                number: None,
                placement: None,
            }),
            mordent: None,
            inverted_mordent: None,
            schleifer: None,
            tremolo: None,
            haydn: None,
            other_ornament: None,
            accidental_marks: vec![],
        })
    } else {
        None
    };

    // Articulations
    let placed = |active| {
        if active {
            Some(PlacedEmpty {
                placement: None,
                default_x: None,
                default_y: None,
            })
        } else {
            None
        }
    };
    let staccato = placed(eff.staccato);
    let accent = placed(eff.accentuated_note);
    let strong_accent = if eff.heavy_accentuated_note {
        Some(musicxml::note::StrongAccent {
            placement: None,
            accent_type: None,
        })
    } else {
        None
    };
    let articulations = if staccato.is_some() || accent.is_some() || strong_accent.is_some() {
        vec![Articulations {
            accent,
            strong_accent,
            staccato,
            tenuto: None,
            detached_legato: None,
            staccatissimo: None,
            spiccato: None,
            scoop: None,
            plop: None,
            doit: None,
            falloff: None,
            breath_mark: None,
            caesura: None,
            stress: None,
            unstress: None,
            soft_accent: None,
            other_articulation: None,
        }]
    } else {
        vec![]
    };

    if technical_opt.is_none()
        && slides.is_empty()
        && ornaments.is_none()
        && articulations.is_empty()
    {
        return vec![];
    }

    vec![Notations {
        print_object: None,
        footnote: None,
        level: None,
        tied: vec![],
        slurs: vec![],
        tuplets: vec![],
        glissandos: vec![],
        slides,
        ornaments,
        technical: technical_opt,
        articulations,
        dynamics: vec![],
        fermatas: vec![],
        arpeggiate: None,
        non_arpeggiate: None,
        accidental_marks: vec![],
        other_notations: vec![],
    }]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::legacy::note::Note;

    /// A note with no effects and no string assignment → empty notations.
    #[test]
    fn no_effects_returns_empty() {
        let note = Note::default();
        let result = build_notations(&note, &[]);
        assert!(result.is_empty());
    }

    /// A note with `hammer = true` produces a `<technical>` block with `<hammer-on>`.
    #[test]
    fn hammer_on_creates_technical() {
        let mut note = Note::default();
        note.effect.hammer = true;
        let result = build_notations(&note, &[]);
        assert_eq!(result.len(), 1);
        assert!(result[0].technical.as_ref().unwrap().hammer_on.is_some());
    }

    /// A note with `staccato = true` produces an `<articulations>` block.
    #[test]
    fn staccato_creates_articulation() {
        let mut note = Note::default();
        note.effect.staccato = true;
        let result = build_notations(&note, &[]);
        assert_eq!(result.len(), 1);
        assert!(!result[0].articulations.is_empty());
        assert!(result[0].articulations[0].staccato.is_some());
    }

    /// `vibrato = true` produces an `<ornaments>` block with a `<wavy-line>`.
    #[test]
    fn vibrato_creates_ornament() {
        let mut note = Note::default();
        note.effect.vibrato = true;
        let result = build_notations(&note, &[]);
        assert_eq!(result.len(), 1);
        assert!(result[0].ornaments.as_ref().unwrap().wavy_line.is_some());
    }

    /// A fretted note (string > 0, valid string index) populates fret + string number.
    #[test]
    fn fretted_note_has_fret_and_string() {
        let note = Note {
            string: 1,
            value: 5, /* fret */
            ..Default::default()
        };
        let strings: Vec<(i8, i8)> = vec![(1, 40)]; // string 1 = E2 (MIDI 40)
        let result = build_notations(&note, &strings);
        assert_eq!(result.len(), 1);
        let tech = result[0].technical.as_ref().unwrap();
        assert_eq!(tech.fret.as_ref().unwrap().value, 5);
        assert_eq!(tech.string.as_ref().unwrap().value, 1);
    }

    /// `let_ring = true` is encoded as `other-technical`.
    #[test]
    fn let_ring_uses_other_technical() {
        let mut note = Note::default();
        note.effect.let_ring = true;
        let result = build_notations(&note, &[]);
        assert_eq!(result.len(), 1);
        let other = result[0]
            .technical
            .as_ref()
            .unwrap()
            .other_technical
            .as_ref()
            .unwrap();
        assert_eq!(other.value.as_deref(), Some("let-ring"));
    }
}
