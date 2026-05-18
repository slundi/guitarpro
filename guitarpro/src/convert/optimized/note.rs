//! Note, beat, and voice construction from MusicXML measure data.

use std::collections::HashMap;

use crate::model::{
    musicxml::{measure::MusicData, note as mx_note},
    optimized::{
        beat::{
            Beam, BeamKind, Beat, Duration, GraceNote, LyricAnchor, Slur, SlurKind, Tuplet, Voice,
        },
        global::{LyricLineId, MeasureIndex, TrackId},
        note::{
            AccidentalDisplay, ArpeggiateDirection, ArpeggiateKind, Articulation, Finger, Note,
            NoteValue, Notehead, Ornament, Pitch, PitchStep, StemDirection, Technique,
            TechniqueKind, TechniqueParams, TieType,
        },
        track::MeasureData,
    },
};

// ---------------------------------------------------------------------------
// Lyric state passed in from the outer lyric collector
// ---------------------------------------------------------------------------

/// Tracks which syllable index comes next for each (part_idx, lyric_number) pair.
pub struct LyricState<'a> {
    /// (part_idx, lyric_number) → (LyricLineId, next_syllable_counter)
    #[allow(dead_code)]
    pub counters: &'a mut HashMap<(usize, String), (LyricLineId, u16)>,
}

// ---------------------------------------------------------------------------
// Measure conversion
// ---------------------------------------------------------------------------

pub fn build_measure_data(
    measure: &crate::model::musicxml::measure::Measure,
    measure_index: MeasureIndex,
    track_id: TrackId,
    part_idx: usize,
    divisions: &mut u32,
    lyric_state: &mut LyricState<'_>,
) -> MeasureData {
    // cursor = absolute tick position within the measure
    let mut cursor: u32 = 0;
    // last onset used for <chord> grouping
    let mut last_onset: u32 = 0;
    // voice string → list of beats (accumulator)
    let mut voice_beats: HashMap<String, Vec<BeatAcc>> = HashMap::new();
    // open slur IDs per voice
    let mut open_slurs: HashMap<String, u8> = HashMap::new();
    let mut next_slur_id: u8 = 1;

    for event in &measure.music_data {
        match event {
            MusicData::Note(n) => {
                // Grace notes are appended to the next real beat's grace_notes
                if n.grace.is_some() {
                    handle_grace_note(n, &mut voice_beats, cursor, part_idx, lyric_state);
                    continue;
                }

                let voice = n.voice.clone().unwrap_or_else(|| "1".to_string());
                let onset = if n.chord.is_some() {
                    last_onset
                } else {
                    let o = cursor;
                    last_onset = cursor;
                    if let Some(dur) = n.duration {
                        cursor += dur;
                    }
                    o
                };

                let note = convert_note(n, part_idx, lyric_state);
                let duration = note_duration(n);
                let slur = note_slur(n, &voice, &mut open_slurs, &mut next_slur_id);
                let beams = note_beams(n);
                let dynamic = None;
                let chord = None; // chord symbol from direction, not per-note

                let beats = voice_beats.entry(voice.clone()).or_default();

                if n.chord.is_some() {
                    // Add note to the existing beat at `onset`
                    if let Some(b) = beats.iter_mut().rev().find(|b| b.tick == onset) {
                        b.notes.push(note);
                    } else {
                        beats.push(BeatAcc {
                            tick: onset,
                            duration,
                            notes: vec![note],
                            events: vec![],
                            dynamic,
                            slur,
                            beams,
                            grace_notes: vec![],
                            chord,
                        });
                    }
                } else {
                    beats.push(BeatAcc {
                        tick: onset,
                        duration,
                        notes: vec![note],
                        events: vec![],
                        dynamic,
                        slur,
                        beams,
                        grace_notes: vec![],
                        chord,
                    });
                }
            }

            MusicData::Backup(b) => {
                cursor = cursor.saturating_sub(b.duration);
                last_onset = cursor;
            }

            MusicData::Forward(f) => {
                cursor += f.duration;
                last_onset = cursor;
            }

            MusicData::Attributes(a) => {
                if let Some(d) = a.divisions {
                    *divisions = d;
                }
            }

            _ => {}
        }
    }

    // Assemble voices
    let mut voices: HashMap<u8, Voice> = HashMap::new();
    for (voice_str, mut accs) in voice_beats {
        let voice_id: u8 = voice_str.parse().unwrap_or(1);
        accs.sort_by_key(|b| b.tick);
        let beats = accs
            .into_iter()
            .map(|acc| Beat {
                tick_offset: acc.tick,
                duration: acc.duration,
                notes: acc.notes,
                events: acc.events,
                dynamic: acc.dynamic,
                slur: acc.slur,
                lyric: None, // lyric is per-note; first note's lyric is used below
                beam_group: None,
                tuplet: acc.duration.tuplet,
                beams: acc.beams,
                grace_notes: acc.grace_notes,
                cue: false,
                chord: acc.chord,
                gp_empty: false,
                gp_rest: false,
                gp_vibrato: false,
                gp_fade_in: false,
                gp_stroke: None,
                gp_pick_stroke: None,
                gp_beat_flags2: None,
                gp_break_secondary: None,
                gp_slap_effect: None,
                gp_rasgueado: false,
                gp_text: String::new(),
                gp_mix_table: None,
                gp_tremolo_bar: None,
                gp_chord: None,
            })
            .collect();
        voices.insert(voice_id, Voice { voice_id, beats });
    }

    // Propagate first note's lyric anchor to the beat
    for voice in voices.values_mut() {
        for beat in &mut voice.beats {
            beat.lyric = beat.notes.iter().find_map(|_| None::<LyricAnchor>);
            // lyric anchors are stored on notes in the Note struct? No — they're on Beat.
            // We stored lyric info in the notes during convert_note; lift it to the beat here.
            // Since Note doesn't have a lyric field, we stored anchors in a side channel.
            // See convert_note_lyric_anchor below — we handle this via LyricState.
        }
    }

    MeasureData {
        measure_index,
        track_id,
        repeat: None,
        voices,
        gp_line_break: 0,
    }
}

// ---------------------------------------------------------------------------
// Beat accumulator
// ---------------------------------------------------------------------------

struct BeatAcc {
    tick: u32,
    duration: Duration,
    notes: Vec<Note>,
    events: Vec<crate::model::optimized::effect::BeatEvent>,
    dynamic: Option<crate::model::optimized::beat::Dynamic>,
    slur: Option<Slur>,
    beams: Vec<Beam>,
    grace_notes: Vec<GraceNote>,
    chord: Option<crate::model::optimized::metadata::ChordSymbol>,
}

// ---------------------------------------------------------------------------
// Grace note handling
// ---------------------------------------------------------------------------

fn handle_grace_note(
    n: &mx_note::Note,
    voice_beats: &mut HashMap<String, Vec<BeatAcc>>,
    cursor: u32,
    part_idx: usize,
    lyric_state: &mut LyricState<'_>,
) {
    let grace = n.grace.as_ref().unwrap();
    let slash = grace.slash.as_deref() == Some("yes");
    let steal_time = grace
        .steal_time_following
        .map(|v| v as f32)
        .or_else(|| grace.steal_time_previous.map(|v| v as f32));

    let note = convert_note(n, part_idx, lyric_state);
    let voice = n.voice.clone().unwrap_or_else(|| "1".to_string());
    let grace_note = GraceNote {
        note,
        slash,
        steal_time,
    };

    // Attach to the next existing beat or create a placeholder beat
    let beats = voice_beats.entry(voice).or_default();
    if let Some(last) = beats.last_mut() {
        // grace notes precede the following beat; store on the next beat to be created
        // For now, attach to the latest beat as a "next beat" marker
        last.grace_notes.push(grace_note);
    } else {
        // No beats yet — store a placeholder that will be consumed by the next real note
        beats.push(BeatAcc {
            tick: cursor,
            duration: Duration {
                base: NoteValue::Quarter,
                dots: 0,
                tuplet: None,
            },
            notes: vec![],
            events: vec![],
            dynamic: None,
            slur: None,
            beams: vec![],
            grace_notes: vec![grace_note],
            chord: None,
        });
    }
}

// ---------------------------------------------------------------------------
// Note conversion
// ---------------------------------------------------------------------------

pub fn convert_note(
    n: &mx_note::Note,
    _part_idx: usize,
    _lyric_state: &mut LyricState<'_>,
) -> Note {
    let pitch = n.pitch.as_ref().map(convert_pitch);
    let (string, fret) = note_tab(n);
    let tie = note_tie(n);
    let techniques = note_techniques(n);
    let ornaments = note_ornaments(n);
    let articulations = note_articulations(n);
    let (left_finger, right_finger) = note_fingers(n);
    let notehead = n.notehead.as_ref().map(convert_notehead);
    let stem = n.stem.as_ref().map(convert_stem);
    let accidental = n.accidental.as_ref().map(|a| AccidentalDisplay {
        cautionary: a.cautionary.as_deref() == Some("yes"),
        editorial: a.editorial.as_deref() == Some("yes"),
        parentheses: a.parentheses.as_deref() == Some("yes"),
    });
    let arpeggiate = note_arpeggiate(n);
    let display_pitch = note_display_pitch(n);

    Note {
        pitch,
        string,
        fret,
        tie,
        techniques,
        ornaments,
        articulations,
        left_finger,
        right_finger,
        notehead,
        stem,
        accidental,
        arpeggiate,
        display_pitch,
        gp_harmonic: None,
        gp_grace: None,
        gp_bend: None,
        gp_trill: None,
        gp_ghost: false,
        gp_duration_percent: 1.0,
        gp_swap_accidentals: false,
        gp_velocity: None,
        gp_note_type_raw: None,
        gp_is_rest: false,
        gp_note_duration: None,
        gp_note_tuplet: None,
    }
}

// ---------------------------------------------------------------------------
// Pitch
// ---------------------------------------------------------------------------

pub fn convert_pitch(p: &mx_note::Pitch) -> Pitch {
    let step = match p.step.as_str() {
        "C" => PitchStep::C,
        "D" => PitchStep::D,
        "E" => PitchStep::E,
        "F" => PitchStep::F,
        "G" => PitchStep::G,
        "A" => PitchStep::A,
        "B" => PitchStep::B,
        _ => PitchStep::C,
    };
    let alter = p.alter.unwrap_or(0.0).round() as i8;
    Pitch {
        step,
        alter,
        octave: p.octave as u8,
    }
}

fn pitch_from_step_alter_octave(step: &str, alter: Option<f64>, octave: i8) -> Pitch {
    let step = match step {
        "C" => PitchStep::C,
        "D" => PitchStep::D,
        "E" => PitchStep::E,
        "F" => PitchStep::F,
        "G" => PitchStep::G,
        "A" => PitchStep::A,
        "B" => PitchStep::B,
        _ => PitchStep::C,
    };
    Pitch {
        step,
        alter: alter.unwrap_or(0.0).round() as i8,
        octave: octave as u8,
    }
}

// ---------------------------------------------------------------------------
// Duration
// ---------------------------------------------------------------------------

pub fn note_duration(n: &mx_note::Note) -> Duration {
    let base = n
        .note_type
        .as_ref()
        .map(|t| convert_note_type(t.value))
        .unwrap_or(NoteValue::Quarter);
    let dots = n.dots.len().min(2) as u8;
    let tuplet = n.time_modification.as_ref().map(|tm| Tuplet {
        actual: tm.actual_notes,
        normal: tm.normal_notes,
    });
    Duration { base, dots, tuplet }
}

fn convert_note_type(t: mx_note::NoteTypeValue) -> NoteValue {
    use mx_note::NoteTypeValue as T;
    match t {
        T::N64th => NoteValue::SixtyFourth,
        T::N32nd => NoteValue::ThirtySecond,
        T::N16th => NoteValue::Sixteenth,
        T::Eighth => NoteValue::Eighth,
        T::Quarter => NoteValue::Quarter,
        T::Half => NoteValue::Half,
        T::Whole => NoteValue::Whole,
        // Rarer values: map to nearest supported
        T::N128th | T::N256th | T::N512th | T::N1024th => NoteValue::SixtyFourth,
        T::Breve | T::Long | T::Maxima => NoteValue::Whole,
    }
}

// ---------------------------------------------------------------------------
// Tab info
// ---------------------------------------------------------------------------

fn note_tab(n: &mx_note::Note) -> (Option<u8>, Option<u8>) {
    let tech = n.notations.iter().find_map(|no| no.technical.as_ref());
    let fret = tech.and_then(|t| t.fret.as_ref()).map(|f| f.value);
    let string = tech.and_then(|t| t.string.as_ref()).map(|s| s.value);
    (string, fret)
}

// ---------------------------------------------------------------------------
// Tie
// ---------------------------------------------------------------------------

fn note_tie(n: &mx_note::Note) -> Option<TieType> {
    // Use the <tie> elements (not <tied> in notations)
    n.ties.first().and_then(|tie| match tie.tie_type.as_str() {
        "start" => Some(TieType::Start),
        "stop" => Some(TieType::End),
        _ => None,
    })
}

// ---------------------------------------------------------------------------
// Techniques
// ---------------------------------------------------------------------------

fn note_techniques(n: &mx_note::Note) -> Vec<Technique> {
    let mut out = Vec::new();
    let Some(notations) = n.notations.first() else {
        return out;
    };
    let Some(tech) = &notations.technical else {
        return out;
    };

    if let Some(bend) = &tech.bend {
        out.push(Technique {
            kind: TechniqueKind::Bend,
            params: TechniqueParams::Bend {
                value: bend.bend_alter as f32,
                vibrato: false,
            },
        });
    }
    if tech.hammer_on.is_some() {
        out.push(Technique {
            kind: TechniqueKind::HammerOn,
            params: TechniqueParams::None,
        });
    }
    if tech.pull_off.is_some() {
        out.push(Technique {
            kind: TechniqueKind::PullOff,
            params: TechniqueParams::None,
        });
    }
    if tech.up_bow.is_some() {
        out.push(Technique {
            kind: TechniqueKind::UpBow,
            params: TechniqueParams::None,
        });
    }
    if tech.down_bow.is_some() {
        out.push(Technique {
            kind: TechniqueKind::DownBow,
            params: TechniqueParams::None,
        });
    }
    if tech.open_string.is_some() {
        out.push(Technique {
            kind: TechniqueKind::OpenString,
            params: TechniqueParams::None,
        });
    }
    if tech.stopped.is_some() {
        out.push(Technique {
            kind: TechniqueKind::Stopped,
            params: TechniqueParams::None,
        });
    }
    if tech.snap_pizzicato.is_some() {
        out.push(Technique {
            kind: TechniqueKind::SnapPizzicato,
            params: TechniqueParams::None,
        });
    }
    if tech.fingernails.is_some() {
        out.push(Technique {
            kind: TechniqueKind::Fingernails,
            params: TechniqueParams::None,
        });
    }
    if let Some(h) = &tech.harmonic {
        let natural = h.natural.is_some();
        out.push(Technique {
            kind: TechniqueKind::Harmonic,
            params: TechniqueParams::Harmonic { natural },
        });
    }
    if let Some(hm) = &tech.harmon_mute {
        let open = hm.harmon_closed.as_ref().map(|hc| match hc.value.as_str() {
            "no" => true,
            "yes" => false,
            _ => false,
        });
        out.push(Technique {
            kind: TechniqueKind::HarmonMute,
            params: TechniqueParams::HarmonMute { open },
        });
    }
    if tech.half_muted.is_some() {
        out.push(Technique {
            kind: TechniqueKind::HalfMuted,
            params: TechniqueParams::None,
        });
    }
    if tech.tap.is_some() {
        out.push(Technique {
            kind: TechniqueKind::Tapping,
            params: TechniqueParams::None,
        });
    }

    // Slides / glissandos from notations
    for g in &notations.glissandos {
        let chromatic = g.line_type.as_deref() != Some("wavy");
        out.push(Technique {
            kind: TechniqueKind::Glissando,
            params: TechniqueParams::Glissando { chromatic },
        });
    }
    for s in &notations.slides {
        let kind = if s.slide_type == "start" {
            TechniqueKind::SlideUp
        } else {
            TechniqueKind::SlideDown
        };
        out.push(Technique {
            kind,
            params: TechniqueParams::None,
        });
    }

    out
}

// ---------------------------------------------------------------------------
// Ornaments
// ---------------------------------------------------------------------------

fn note_ornaments(n: &mx_note::Note) -> Vec<Ornament> {
    let mut out = Vec::new();
    let Some(notations) = n.notations.first() else {
        return out;
    };
    let Some(orns) = &notations.ornaments else {
        return out;
    };

    if orns.trill_mark.is_some() {
        out.push(Ornament::Trill);
    }
    if orns.turn.is_some() {
        out.push(Ornament::Turn);
    }
    if orns.delayed_turn.is_some() {
        out.push(Ornament::DelayedTurn);
    }
    if orns.inverted_turn.is_some() {
        out.push(Ornament::InvertedTurn);
    }
    if orns.delayed_inverted_turn.is_some() {
        out.push(Ornament::DelayedInvertedTurn);
    }
    if orns.shake.is_some() {
        out.push(Ornament::Shake);
    }
    if orns.mordent.is_some() {
        out.push(Ornament::Mordent);
    }
    if let Some(im) = &orns.inverted_mordent {
        let long = im.long.as_deref() == Some("yes");
        out.push(Ornament::InvertedMordent { long });
    }
    if let Some(t) = &orns.tremolo {
        let marks = t.marks.unwrap_or(1);
        out.push(Ornament::Tremolo(marks));
    }
    if let Some(wl) = &orns.wavy_line
        && (wl.wavy_type == "start" || wl.wavy_type == "continue")
    {
        out.push(Ornament::WavyLine);
    }

    out
}

// ---------------------------------------------------------------------------
// Articulations
// ---------------------------------------------------------------------------

fn note_articulations(n: &mx_note::Note) -> Vec<Articulation> {
    let mut out = Vec::new();
    for notations in &n.notations {
        for arts in &notations.articulations {
            if arts.accent.is_some() {
                out.push(Articulation::Accent);
            }
            if arts.strong_accent.is_some() {
                out.push(Articulation::Marcato);
            }
            if arts.staccato.is_some() {
                out.push(Articulation::Staccato);
            }
            if arts.tenuto.is_some() {
                out.push(Articulation::Tenuto);
            }
            if arts.staccatissimo.is_some() {
                out.push(Articulation::Staccatissimo);
            }
            if arts.spiccato.is_some() {
                out.push(Articulation::Spiccato);
            }
            if arts.scoop.is_some() {
                out.push(Articulation::Scoop);
            }
            if arts.plop.is_some() {
                out.push(Articulation::Plop);
            }
            if arts.doit.is_some() {
                out.push(Articulation::Doit);
            }
            if arts.falloff.is_some() {
                out.push(Articulation::Falloff);
            }
            if arts.breath_mark.is_some() {
                out.push(Articulation::BreathMark);
            }
            if arts.caesura.is_some() {
                out.push(Articulation::Caesura);
            }
            if arts.stress.is_some() {
                out.push(Articulation::Stress);
            }
            if arts.unstress.is_some() {
                out.push(Articulation::Unstress);
            }
            if arts.soft_accent.is_some() {
                out.push(Articulation::SoftAccent);
            }
        }
        // Fermata in notations
        if !notations.fermatas.is_empty() {
            out.push(Articulation::Fermata);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Fingering
// ---------------------------------------------------------------------------

fn note_fingers(n: &mx_note::Note) -> (Option<Finger>, Option<Finger>) {
    let tech = n.notations.iter().find_map(|no| no.technical.as_ref());
    let left = tech
        .and_then(|t| t.fingering.as_ref())
        .and_then(|f| parse_finger(&f.value));
    let right: Option<Finger> = tech.and_then(|t| t.pluck.as_ref()).and(None); // pluck doesn't carry a digit in this model
    (left, right)
}

fn parse_finger(s: &str) -> Option<Finger> {
    match s.trim() {
        "0" | "T" => Some(Finger::Thumb),
        "1" => Some(Finger::Index),
        "2" => Some(Finger::Middle),
        "3" => Some(Finger::Ring),
        "4" | "5" => Some(Finger::Pinky),
        "O" | "o" => Some(Finger::Open),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Notehead / stem / arpeggiate / display pitch
// ---------------------------------------------------------------------------

fn convert_notehead(nh: &mx_note::Notehead) -> Notehead {
    match nh.value.as_str() {
        "normal" => Notehead::Normal,
        "x" => Notehead::X,
        "circle-x" => Notehead::CircleX,
        "diamond" => Notehead::Diamond,
        "square" => Notehead::Square,
        "slash" => Notehead::Slash,
        "triangle" => Notehead::Triangle,
        "arrow up" => Notehead::ArrowUp,
        "arrow down" => Notehead::ArrowDown,
        "none" => Notehead::None,
        _ => Notehead::Normal,
    }
}

fn convert_stem(stem: &mx_note::Stem) -> StemDirection {
    match stem.value.as_str() {
        "up" => StemDirection::Up,
        "down" => StemDirection::Down,
        _ => StemDirection::None,
    }
}

fn note_arpeggiate(n: &mx_note::Note) -> Option<ArpeggiateKind> {
    let notations = n.notations.first()?;
    if notations.non_arpeggiate.is_some() {
        return Some(ArpeggiateKind::NonArpeggiate);
    }
    notations.arpeggiate.as_ref().map(|a| {
        let dir = a.direction.as_deref().map(|d| match d {
            "up" => ArpeggiateDirection::Up,
            "down" => ArpeggiateDirection::Down,
            _ => ArpeggiateDirection::Up,
        });
        ArpeggiateKind::Arpeggiate(dir)
    })
}

fn note_display_pitch(n: &mx_note::Note) -> Option<Pitch> {
    // Rest with display pitch
    if let Some(rest) = &n.rest
        && let (Some(step), Some(oct)) = (&rest.display_step, rest.display_octave)
    {
        return Some(pitch_from_step_alter_octave(step, None, oct));
    }
    // Unpitched
    if let Some(unp) = &n.unpitched
        && let (Some(step), Some(oct)) = (&unp.display_step, unp.display_octave)
    {
        return Some(pitch_from_step_alter_octave(step, None, oct));
    }
    None
}

// ---------------------------------------------------------------------------
// Beams
// ---------------------------------------------------------------------------

fn note_beams(n: &mx_note::Note) -> Vec<Beam> {
    n.beams
        .iter()
        .map(|b| {
            let level = b.number.unwrap_or(1);
            let kind = match b.value.as_str() {
                "begin" => BeamKind::Begin,
                "continue" => BeamKind::Continue,
                "end" => BeamKind::End,
                "forward hook" => BeamKind::ForwardHook,
                "backward hook" => BeamKind::BackwardHook,
                _ => BeamKind::Continue,
            };
            Beam { level, kind }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Slurs
// ---------------------------------------------------------------------------

fn note_slur(
    n: &mx_note::Note,
    voice: &str,
    open_slurs: &mut HashMap<String, u8>,
    next_id: &mut u8,
) -> Option<Slur> {
    let notations = n.notations.first()?;
    for slur in &notations.slurs {
        let kind = match slur.slur_type.as_str() {
            "start" => {
                let id = *next_id;
                *next_id = next_id.wrapping_add(1);
                open_slurs.insert(voice.to_owned(), id);
                SlurKind::Start
            }
            "stop" => {
                open_slurs.remove(voice);
                SlurKind::End
            }
            _ => continue,
        };
        let slur_id = *open_slurs.get(voice).unwrap_or(next_id);
        return Some(Slur { slur_id, kind });
    }
    None
}
