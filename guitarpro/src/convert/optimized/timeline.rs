//! Build the global measure timeline from a `ScorePartwise`.

use crate::model::{
    musicxml::{ScorePartwise, measure::MusicData},
    optimized::{
        global::MeasureIndex,
        metadata::{KeySignature, Mode, TimeSignature},
        note::{Pitch, PitchStep},
        timeline::{
            Barline, BarlineStyle, Ending, EndingKind, JumpKind, MeasureDef, NavigationEvent,
        },
    },
};

pub const DEFAULT_DIVISIONS: u32 = 480;
pub const DEFAULT_TEMPO: f32 = 120.0;

pub struct TimelineData {
    pub measures: Vec<MeasureDef>,
    #[allow(dead_code)]
    pub initial_divisions: u32,
    pub initial_tempo: f32,
    pub initial_time_sig: TimeSignature,
    pub initial_key_sig: KeySignature,
}

pub fn build_timeline(src: &ScorePartwise) -> TimelineData {
    let default_time_sig = TimeSignature {
        numerator: 4,
        denominator: 4,
    };
    let default_key_sig = KeySignature {
        root: Pitch {
            step: PitchStep::C,
            alter: 0,
            octave: 4,
        },
        mode: Mode::Major,
    };

    let Some(first_part) = src.parts.first() else {
        return TimelineData {
            measures: Vec::new(),
            initial_divisions: DEFAULT_DIVISIONS,
            initial_tempo: DEFAULT_TEMPO,
            initial_time_sig: default_time_sig,
            initial_key_sig: default_key_sig,
        };
    };

    let mut divisions: u32 = DEFAULT_DIVISIONS;
    let mut tempo: f32 = DEFAULT_TEMPO;
    let mut time_sig = default_time_sig;
    let mut key_sig = default_key_sig;

    let mut initial_divisions = DEFAULT_DIVISIONS;
    let mut initial_tempo = DEFAULT_TEMPO;
    let mut initial_time_sig = default_time_sig;
    let mut initial_key_sig = default_key_sig;
    let mut captured_initial = false;

    let mut measures: Vec<MeasureDef> = Vec::new();

    for (measure_idx, measure) in first_part.measures.iter().enumerate() {
        let mut measure_tempo: Option<f32> = None;
        let mut measure_time_sig: Option<TimeSignature> = None;
        let mut measure_key_sig: Option<KeySignature> = None;
        let mut barline_left: Option<Barline> = None;
        let mut barline_right: Option<Barline> = None;
        let mut navigation: Vec<NavigationEvent> = Vec::new();

        for event in &measure.music_data {
            match event {
                MusicData::Attributes(attrs) => {
                    if let Some(d) = attrs.divisions {
                        divisions = d;
                    }
                    if let Some(time) = attrs.times.first() {
                        if let (Some(b), Some(bt)) = (&time.beats, &time.beat_type) {
                            if let (Ok(num), Ok(den)) = (b.parse::<u8>(), bt.parse::<u8>()) {
                                let ts = TimeSignature {
                                    numerator: num,
                                    denominator: den,
                                };
                                if measure_idx == 0 || time_sig != ts {
                                    measure_time_sig = Some(ts);
                                }
                                time_sig = ts;
                            }
                        }
                    }
                    if let Some(key) = attrs.keys.first() {
                        if let Some(fifths) = key.fifths {
                            let ks = key_sig_from_fifths(fifths, key.mode.as_deref());
                            if measure_idx == 0 || key_sig != ks {
                                measure_key_sig = Some(ks);
                            }
                            key_sig = ks;
                        }
                    }
                }

                MusicData::Direction(dir) => {
                    if let Some(sound) = &dir.sound {
                        if let Some(t) = sound.tempo {
                            let t = t as f32;
                            if t != tempo {
                                measure_tempo = Some(t);
                            }
                            tempo = t;
                        }
                    }
                    for dt_wrapper in &dir.direction_types {
                        use crate::model::musicxml::direction::DirectionType;
                        if let DirectionType::Metronome(metro) = &dt_wrapper.content {
                            if let Some(pm) = &metro.per_minute {
                                if let Ok(bpm) = pm.value.parse::<f32>() {
                                    if bpm != tempo {
                                        measure_tempo = Some(bpm);
                                    }
                                    tempo = bpm;
                                }
                            }
                        }
                    }
                }

                MusicData::Sound(sound) => {
                    if let Some(t) = sound.tempo {
                        let t = t as f32;
                        if t != tempo {
                            measure_tempo = Some(t);
                        }
                        tempo = t;
                    }
                }

                MusicData::Barline(bl) => {
                    let location = bl.location.as_deref().unwrap_or("right");
                    let converted = convert_barline(bl, measure_idx as u16, &mut navigation);
                    match location {
                        "left" => barline_left = converted,
                        _ => barline_right = converted,
                    }
                }

                _ => {}
            }
        }

        if !captured_initial {
            initial_divisions = divisions;
            initial_tempo = tempo;
            initial_time_sig = time_sig;
            initial_key_sig = key_sig;
            // Always emit initial values on measure 0
            if measure_tempo.is_none() {
                measure_tempo = Some(tempo);
            }
            if measure_time_sig.is_none() {
                measure_time_sig = Some(time_sig);
            }
            if measure_key_sig.is_none() {
                measure_key_sig = Some(key_sig);
            }
            captured_initial = true;
        }

        let denominator = time_sig.denominator as u32;
        let duration_ticks = if denominator > 0 {
            time_sig.numerator as u32 * (divisions * 4 / denominator)
        } else {
            divisions * 4
        };

        measures.push(MeasureDef {
            index: MeasureIndex(measure_idx as u16),
            tempo: measure_tempo,
            time_signature: measure_time_sig,
            key_signature: measure_key_sig,
            marker: None,
            navigation,
            tick_resolution: divisions.min(u16::MAX as u32) as u16,
            duration_ticks,
            barline_left,
            barline_right,
        });
    }

    TimelineData {
        measures,
        initial_divisions,
        initial_tempo,
        initial_time_sig,
        initial_key_sig,
    }
}

// ---------------------------------------------------------------------------
// Barline conversion
// ---------------------------------------------------------------------------

fn convert_barline(
    bl: &crate::model::musicxml::barline::Barline,
    measure_idx: u16,
    navigation: &mut Vec<NavigationEvent>,
) -> Option<Barline> {
    // Repeat marks → navigation events
    if let Some(repeat) = &bl.repeat {
        let kind = if repeat.direction == "forward" {
            JumpKind::RepeatOpen
        } else {
            JumpKind::RepeatClose
        };
        let repeat_count = if kind == JumpKind::RepeatClose {
            Some(repeat.times.unwrap_or(2))
        } else {
            None
        };
        navigation.push(NavigationEvent {
            measure_index: MeasureIndex(measure_idx),
            kind,
            repeat_count,
            volta: None,
            volta_last: false,
        });
    }

    // Volta endings: tag onto last RepeatClose or emit standalone
    if let Some(ending) = &bl.ending {
        let numbers: Vec<u8> = ending
            .number
            .split(',')
            .filter_map(|s| s.trim().parse::<u8>().ok())
            .collect();
        let first_volta = numbers.first().copied();
        let kind_str = ending.ending_type.as_str();
        let last = kind_str == "stop" || kind_str == "discontinue";

        if let Some(nav) = navigation.last_mut() {
            if nav.kind == JumpKind::RepeatClose {
                nav.volta = first_volta;
                nav.volta_last = last;
            }
        } else if kind_str == "stop" || kind_str == "discontinue" {
            // Last volta bracket with no repeat close
            navigation.push(NavigationEvent {
                measure_index: MeasureIndex(measure_idx),
                kind: JumpKind::RepeatClose,
                repeat_count: None,
                volta: first_volta,
                volta_last: true,
            });
        }
    }

    // Visual barline style
    bl.bar_style.as_ref().map(|bs| {
        let style = convert_bar_style(bs.value);
        let ending = bl.ending.as_ref().map(|e| {
            let numbers: Vec<u8> = e
                .number
                .split(',')
                .filter_map(|s| s.trim().parse::<u8>().ok())
                .collect();
            let kind = match e.ending_type.as_str() {
                "start" => EndingKind::Start,
                "discontinue" => EndingKind::Discontinue,
                _ => EndingKind::Stop,
            };
            Ending {
                numbers,
                text: e.value.clone(),
                kind,
            }
        });
        Barline { style, ending }
    })
}

fn convert_bar_style(style: crate::model::musicxml::barline::BarStyle) -> BarlineStyle {
    use crate::model::musicxml::barline::BarStyle;
    match style {
        BarStyle::Regular => BarlineStyle::Regular,
        BarStyle::Dotted => BarlineStyle::Dotted,
        BarStyle::Dashed => BarlineStyle::Dashed,
        BarStyle::Heavy => BarlineStyle::Heavy,
        BarStyle::LightLight => BarlineStyle::LightLight,
        BarStyle::LightHeavy => BarlineStyle::LightHeavy,
        BarStyle::HeavyLight => BarlineStyle::HeavyLight,
        BarStyle::HeavyHeavy => BarlineStyle::HeavyHeavy,
        BarStyle::Tick => BarlineStyle::Tick,
        BarStyle::Short => BarlineStyle::Short,
        BarStyle::None => BarlineStyle::None,
    }
}

// ---------------------------------------------------------------------------
// Key signature conversion: fifths → root + mode
// ---------------------------------------------------------------------------

pub fn key_sig_from_fifths(fifths: i8, mode: Option<&str>) -> KeySignature {
    let mode_val = match mode {
        Some("minor") | Some("aeolian") => Mode::Minor,
        Some("dorian") => Mode::Dorian,
        Some("phrygian") => Mode::Phrygian,
        Some("lydian") => Mode::Lydian,
        Some("mixolydian") => Mode::Mixolydian,
        Some("locrian") => Mode::Locrian,
        _ => Mode::Major,
    };

    let root = if mode_val == Mode::Minor {
        // Relative minor (sixth degree of the major key sharing this key signature)
        match fifths {
            -7 => Pitch {
                step: PitchStep::A,
                alter: -1,
                octave: 4,
            },
            -6 => Pitch {
                step: PitchStep::E,
                alter: -1,
                octave: 4,
            },
            -5 => Pitch {
                step: PitchStep::B,
                alter: -1,
                octave: 4,
            },
            -4 => Pitch {
                step: PitchStep::F,
                alter: 0,
                octave: 4,
            },
            -3 => Pitch {
                step: PitchStep::C,
                alter: 0,
                octave: 4,
            },
            -2 => Pitch {
                step: PitchStep::G,
                alter: 0,
                octave: 4,
            },
            -1 => Pitch {
                step: PitchStep::D,
                alter: 0,
                octave: 4,
            },
            0 => Pitch {
                step: PitchStep::A,
                alter: 0,
                octave: 4,
            },
            1 => Pitch {
                step: PitchStep::E,
                alter: 0,
                octave: 4,
            },
            2 => Pitch {
                step: PitchStep::B,
                alter: 0,
                octave: 4,
            },
            3 => Pitch {
                step: PitchStep::F,
                alter: 1,
                octave: 4,
            },
            4 => Pitch {
                step: PitchStep::C,
                alter: 1,
                octave: 4,
            },
            5 => Pitch {
                step: PitchStep::G,
                alter: 1,
                octave: 4,
            },
            6 => Pitch {
                step: PitchStep::D,
                alter: 1,
                octave: 4,
            },
            7 => Pitch {
                step: PitchStep::A,
                alter: 1,
                octave: 4,
            },
            _ => Pitch {
                step: PitchStep::A,
                alter: 0,
                octave: 4,
            },
        }
    } else {
        // Major root
        match fifths {
            -7 => Pitch {
                step: PitchStep::C,
                alter: -1,
                octave: 4,
            },
            -6 => Pitch {
                step: PitchStep::G,
                alter: -1,
                octave: 4,
            },
            -5 => Pitch {
                step: PitchStep::D,
                alter: -1,
                octave: 4,
            },
            -4 => Pitch {
                step: PitchStep::A,
                alter: -1,
                octave: 4,
            },
            -3 => Pitch {
                step: PitchStep::E,
                alter: -1,
                octave: 4,
            },
            -2 => Pitch {
                step: PitchStep::B,
                alter: -1,
                octave: 4,
            },
            -1 => Pitch {
                step: PitchStep::F,
                alter: 0,
                octave: 4,
            },
            0 => Pitch {
                step: PitchStep::C,
                alter: 0,
                octave: 4,
            },
            1 => Pitch {
                step: PitchStep::G,
                alter: 0,
                octave: 4,
            },
            2 => Pitch {
                step: PitchStep::D,
                alter: 0,
                octave: 4,
            },
            3 => Pitch {
                step: PitchStep::A,
                alter: 0,
                octave: 4,
            },
            4 => Pitch {
                step: PitchStep::E,
                alter: 0,
                octave: 4,
            },
            5 => Pitch {
                step: PitchStep::B,
                alter: 0,
                octave: 4,
            },
            6 => Pitch {
                step: PitchStep::F,
                alter: 1,
                octave: 4,
            },
            7 => Pitch {
                step: PitchStep::C,
                alter: 1,
                octave: 4,
            },
            _ => Pitch {
                step: PitchStep::C,
                alter: 0,
                octave: 4,
            },
        }
    };

    KeySignature {
        root,
        mode: mode_val,
    }
}
