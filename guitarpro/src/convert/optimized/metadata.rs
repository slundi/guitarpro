//! Score metadata, instrument definitions, part groups, and lyric collection.

use std::collections::HashMap;

type OpenGroupMap = HashMap<
    String,
    (
        Option<String>,
        Option<String>,
        GroupSymbol,
        GroupBarline,
        usize,
    ),
>;

use crate::model::{
    musicxml::{ScorePartwise, measure::MusicData, part_list::PartListItem},
    optimized::{
        display::ScoreDefaults,
        global::{
            GroupBarline, GroupSymbol, Instrument, InstrumentKind, LyricLine, LyricLineId,
            LyricProjection, LyricSyllable, PartGroup, TrackId, Transpose,
        },
        metadata::{Creator, Credit, Identification, Metadata, TextJustify, TextValign},
        note::Pitch,
    },
};

use super::timeline::TimelineData;

// ---------------------------------------------------------------------------
// Metadata
// ---------------------------------------------------------------------------

pub fn build_metadata(src: &ScorePartwise, tl: &TimelineData) -> Metadata {
    let title = src
        .work
        .as_ref()
        .and_then(|w| w.work_title.clone())
        .or_else(|| src.movement_title.clone())
        .unwrap_or_default();

    let work = src
        .work
        .as_ref()
        .map(|w| crate::model::optimized::metadata::Work {
            number: w.work_number.clone(),
            title: w.work_title.clone(),
            opus: w.opus.as_ref().and_then(|o| o.href.clone()),
        });

    let movement_number = src.movement_number.clone();

    let identification = src.identification.as_ref().map(build_identification);

    let (artist, composer, album, copyright) = extract_common_fields(src);

    let credits = build_credits(src);

    Metadata {
        work,
        movement_number,
        title,
        artist,
        album,
        composer,
        year: None,
        copyright,
        identification,
        credits,
        master_tempo: tl.initial_tempo,
        time_signature: tl.initial_time_sig,
        key_signature: tl.initial_key_sig,
        chords: vec![],
        scale_hint: None,
    }
}

fn extract_common_fields(
    src: &ScorePartwise,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    let mut artist: Option<String> = None;
    let mut composer: Option<String> = None;
    let mut copyright: Option<String> = None;

    if let Some(id) = &src.identification {
        for creator in &id.creators {
            match creator.creator_type.as_deref() {
                Some("composer") => composer = Some(creator.value.clone()),
                Some("lyricist") | Some("arranger") => {}
                _ => {
                    if artist.is_none() {
                        artist = Some(creator.value.clone());
                    }
                }
            }
        }
        for rights in &id.rights {
            if copyright.is_none() {
                copyright = Some(rights.value.clone());
            }
        }
    }

    (artist, composer, None, copyright)
}

fn build_identification(
    id: &crate::model::musicxml::identification::Identification,
) -> Identification {
    let creators = id
        .creators
        .iter()
        .map(|c| Creator {
            role: c
                .creator_type
                .clone()
                .unwrap_or_else(|| "contributor".to_string()),
            name: c.value.clone(),
        })
        .collect();

    let rights = id.rights.first().map(|r| r.value.clone());

    let (encoding_software, encoding_date) = id
        .encoding
        .as_ref()
        .map(|enc| {
            let sw = enc.software.first().cloned();
            let date = enc.encoding_date.clone();
            (sw, date)
        })
        .unwrap_or((None, None));

    let miscellaneous = id
        .miscellaneous
        .as_ref()
        .map(|m| {
            m.fields
                .iter()
                .map(|f| (f.name.clone(), f.value.clone()))
                .collect()
        })
        .unwrap_or_default();

    Identification {
        creators,
        rights,
        encoding_software,
        encoding_date,
        source: id.source.clone(),
        miscellaneous,
    }
}

fn build_credits(src: &ScorePartwise) -> Vec<Credit> {
    use crate::model::musicxml::credit::CreditContent;
    src.credits
        .iter()
        .filter_map(|c| {
            // Find the first CreditWords content item.
            let text = c.content.iter().find_map(|item| {
                if let CreditContent::CreditWords(cw) = item {
                    Some(cw)
                } else {
                    None
                }
            })?;
            // Extract credit type from content.
            let credit_type = c.content.iter().find_map(|item| {
                if let CreditContent::CreditType(s) = item {
                    Some(s.clone())
                } else {
                    None
                }
            });
            let justify = text.justify.as_deref().and_then(|j| match j {
                "left" => Some(TextJustify::Left),
                "center" => Some(TextJustify::Center),
                "right" => Some(TextJustify::Right),
                _ => None,
            });
            let valign = text.valign.as_deref().and_then(|v| match v {
                "top" => Some(TextValign::Top),
                "middle" => Some(TextValign::Middle),
                "bottom" => Some(TextValign::Bottom),
                _ => None,
            });
            let font_size = text
                .font_size
                .as_deref()
                .and_then(|s| s.parse::<f32>().ok());
            Some(Credit {
                credit_type,
                text: text.value.clone(),
                position_x: text.default_x.map(|v| v as f32),
                position_y: text.default_y.map(|v| v as f32),
                font_size,
                justify,
                valign,
                page: c.page.map(|p| p as u16),
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Instruments
// ---------------------------------------------------------------------------

pub fn build_instruments(
    src: &ScorePartwise,
    score_parts: &[&crate::model::musicxml::part_list::ScorePart],
) -> Vec<Instrument> {
    score_parts
        .iter()
        .enumerate()
        .map(|(part_idx, sp)| {
            let name = sp
                .part_name
                .as_ref()
                .and_then(|n| n.value.clone())
                .unwrap_or_else(|| format!("Part {}", part_idx + 1));

            let abbreviation = sp.part_abbreviation.as_ref().and_then(|n| n.value.clone());

            let instrument_sound = sp
                .score_instruments
                .first()
                .and_then(|si| si.instrument_sound.clone());

            let midi = sp.midi_instruments.first();
            let midi_channel = midi
                .and_then(|m| m.midi_channel)
                .map(|c| c.saturating_sub(1)) // MusicXML is 1-based; optimized is 0-based
                .unwrap_or(0);
            let midi_program = midi
                .and_then(|m| m.midi_program)
                .map(|p| p.saturating_sub(1)) // MusicXML is 1-based
                .unwrap_or(0);
            let midi_bank = midi.and_then(|m| m.midi_bank);
            let volume = midi.and_then(|m| m.volume).map(|v| (v / 100.0) as f32);
            let pan = midi
                .and_then(|m| m.pan)
                .map(|p| (p / 90.0).clamp(-1.0, 1.0) as f32);

            let transpose = first_transpose(src, part_idx);
            let kind = instrument_kind(src, part_idx, sp);

            Instrument {
                name,
                abbreviation,
                instrument_sound,
                midi_channel,
                midi_program,
                midi_bank,
                volume,
                pan,
                kind,
                transpose,
                gp_strings: Vec::new(),
            }
        })
        .collect()
}

fn instrument_kind(
    src: &ScorePartwise,
    part_idx: usize,
    sp: &crate::model::musicxml::part_list::ScorePart,
) -> InstrumentKind {
    // Check instrument sound for percussion
    let is_percussion = sp
        .score_instruments
        .first()
        .and_then(|si| si.instrument_sound.as_deref())
        .map(|s| s.contains("drum") || s.contains("percussion") || s.contains("cymbal"))
        .unwrap_or(false);

    if is_percussion {
        return InstrumentKind::Percussion;
    }

    // Check MIDI channel 9 (0-based) → percussion
    let midi_channel_0based = sp
        .midi_instruments
        .first()
        .and_then(|m| m.midi_channel)
        .map(|c| c.saturating_sub(1))
        .unwrap_or(0);
    if midi_channel_0based == 9 {
        return InstrumentKind::Percussion;
    }

    // Look for TAB staff in the part's attributes
    let Some(part) = src.parts.get(part_idx) else {
        return InstrumentKind::Pitched;
    };

    let mut tuning_pitches: Option<Vec<Pitch>> = None;
    let mut capo: u8 = 0;

    'outer: for measure in &part.measures {
        for event in &measure.music_data {
            if let MusicData::Attributes(attrs) = event {
                for sd in &attrs.staff_details {
                    if sd.staff_type.as_deref() == Some("tab") || !sd.staff_tunings.is_empty() {
                        capo = sd.capo.unwrap_or(0);
                        if !sd.staff_tunings.is_empty() {
                            let mut tunings: Vec<(u8, Pitch)> = sd
                                .staff_tunings
                                .iter()
                                .map(|t| {
                                    (
                                        t.line,
                                        Pitch {
                                            step: match t.tuning_step.as_str() {
                                                "C" => crate::model::optimized::note::PitchStep::C,
                                                "D" => crate::model::optimized::note::PitchStep::D,
                                                "E" => crate::model::optimized::note::PitchStep::E,
                                                "F" => crate::model::optimized::note::PitchStep::F,
                                                "G" => crate::model::optimized::note::PitchStep::G,
                                                "A" => crate::model::optimized::note::PitchStep::A,
                                                "B" => crate::model::optimized::note::PitchStep::B,
                                                _ => crate::model::optimized::note::PitchStep::E,
                                            },
                                            alter: t.tuning_alter.unwrap_or(0.0).round() as i8,
                                            octave: t.tuning_octave as u8,
                                        },
                                    )
                                })
                                .collect();
                            tunings.sort_by_key(|(line, _)| *line);
                            tuning_pitches = Some(tunings.into_iter().map(|(_, p)| p).collect());
                        }
                        break 'outer;
                    }
                }
            }
        }
    }

    if let Some(tuning) = tuning_pitches {
        let string_count = tuning.len() as u8;
        return InstrumentKind::Stringed {
            tuning,
            string_count,
            capo,
        };
    }

    InstrumentKind::Pitched
}

fn first_transpose(src: &ScorePartwise, part_idx: usize) -> Option<Transpose> {
    let part = src.parts.get(part_idx)?;
    for measure in &part.measures {
        for event in &measure.music_data {
            if let MusicData::Attributes(attrs) = event
                && let Some(t) = attrs.transposes.first()
            {
                return Some(Transpose {
                    diatonic: t.diatonic,
                    chromatic: t.chromatic,
                    octave_change: t.octave_change,
                });
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Part groups
// ---------------------------------------------------------------------------

pub fn build_groups(src: &ScorePartwise) -> Vec<PartGroup> {
    let mut groups: Vec<PartGroup> = Vec::new();
    // Track open groups: number string → (label, abbreviation, symbol, barline, start_part_idx)
    let mut open: OpenGroupMap = HashMap::new();
    let mut part_counter: usize = 0;

    for item in &src.part_list.items {
        match item {
            PartListItem::PartGroup(pg) => {
                let num = pg.number.clone().unwrap_or_else(|| "1".to_string());
                if pg.group_type == "start" {
                    let label = pg.group_name.as_ref().and_then(|n| n.value.clone());
                    let abbreviation = pg.group_abbreviation.as_ref().and_then(|n| n.value.clone());
                    let symbol = pg
                        .group_symbol
                        .as_ref()
                        .map(|gs| match gs.value.as_str() {
                            "bracket" => GroupSymbol::Bracket,
                            "brace" => GroupSymbol::Brace,
                            "square" => GroupSymbol::Square,
                            "line" => GroupSymbol::Line,
                            _ => GroupSymbol::None,
                        })
                        .unwrap_or(GroupSymbol::None);
                    let barline = pg
                        .group_barline
                        .as_ref()
                        .map(|gb| match gb.value.as_str() {
                            "yes" => GroupBarline::Yes,
                            "Mensurstrich" => GroupBarline::Mensurstrich,
                            _ => GroupBarline::No,
                        })
                        .unwrap_or(GroupBarline::Yes);
                    open.insert(num, (label, abbreviation, symbol, barline, part_counter));
                } else {
                    // "stop"
                    if let Some((label, abbreviation, symbol, barline, start)) = open.remove(&num) {
                        let tracks: Vec<_> =
                            (start..part_counter).map(|i| TrackId(i as u8)).collect();
                        groups.push(PartGroup {
                            label,
                            abbreviation,
                            symbol,
                            barline,
                            tracks,
                        });
                    }
                }
            }
            PartListItem::ScorePart(_) => {
                part_counter += 1;
            }
        }
    }

    groups
}

// ---------------------------------------------------------------------------
// Lyric pre-pass
// ---------------------------------------------------------------------------

pub struct LyricCollection {
    pub lines: Vec<LyricLine>,
    pub projections: Vec<LyricProjection>,
    /// (part_idx, lyric_number_str) → (LyricLineId, next_syllable_counter for main pass)
    pub counters: HashMap<(usize, String), (LyricLineId, u16)>,
}

pub fn collect_lyrics(src: &ScorePartwise) -> LyricCollection {
    let mut lines: Vec<LyricLine> = Vec::new();
    let mut projections: Vec<LyricProjection> = Vec::new();
    let mut line_map: HashMap<(usize, String), LyricLineId> = HashMap::new();

    for (part_idx, part) in src.parts.iter().enumerate() {
        for measure in &part.measures {
            for event in &measure.music_data {
                let MusicData::Note(n) = event else { continue };
                for lyric in &n.lyrics {
                    let lyric_num = lyric.number.clone().unwrap_or_else(|| "1".to_string());
                    let key = (part_idx, lyric_num.clone());

                    let line_id = if let Some(id) = line_map.get(&key) {
                        *id
                    } else {
                        let id = LyricLineId(lines.len() as u8);
                        line_map.insert(key.clone(), id);
                        lines.push(LyricLine {
                            label: lyric.name.clone(),
                            language: None,
                            syllables: Vec::new(),
                        });
                        projections.push(LyricProjection {
                            lyric_line_id: id,
                            anchor_track: TrackId(part_idx as u8),
                            display_track: TrackId(part_idx as u8),
                        });
                        id
                    };

                    let text = lyric
                        .text
                        .as_ref()
                        .map(|t| t.value.clone())
                        .unwrap_or_default();
                    let syllabic = lyric.syllabic.as_deref();
                    let hyphen = matches!(syllabic, Some("begin") | Some("middle"));
                    let elision = lyric.elision.as_ref().and_then(|e| e.value.clone());

                    lines[line_id.0 as usize].syllables.push(LyricSyllable {
                        text,
                        hyphen,
                        line_break: lyric.end_line.is_some(),
                        elision,
                        extend: lyric.extend.is_some(),
                        laughing: lyric.laughing.is_some(),
                        humming: lyric.humming.is_some(),
                    });
                }
            }
        }
    }

    // Build counters initialised to 0 for use in the main pass
    let counters = line_map
        .into_iter()
        .map(|(key, id)| (key, (id, 0u16)))
        .collect();

    LyricCollection {
        lines,
        projections,
        counters,
    }
}

// ---------------------------------------------------------------------------
// Score defaults (from MusicXML <defaults>)
// ---------------------------------------------------------------------------

pub fn build_defaults(src: &ScorePartwise) -> Option<ScoreDefaults> {
    let defaults = src.defaults.as_ref()?;

    let scaling = defaults
        .scaling
        .as_ref()
        .map(|s| crate::model::optimized::display::Scaling {
            millimeters: s.millimeters as f32,
            tenths: s.tenths as f32,
        });

    let page_layout = defaults.page_layout.as_ref().map(|pl| {
        use crate::model::optimized::display::PageLayout;
        let margins = pl.page_margins.first();
        PageLayout {
            width_mm: pl.size.as_ref().map(|s| s.page_width).unwrap_or(0.0) as f32,
            height_mm: pl.size.as_ref().map(|s| s.page_height).unwrap_or(0.0) as f32,
            margin_top: margins.map(|m| m.top_margin).unwrap_or(0.0) as f32,
            margin_bottom: margins.map(|m| m.bottom_margin).unwrap_or(0.0) as f32,
            margin_left: margins.map(|m| m.left_margin).unwrap_or(0.0) as f32,
            margin_right: margins.map(|m| m.right_margin).unwrap_or(0.0) as f32,
        }
    });

    let music_font =
        defaults
            .music_font
            .as_ref()
            .map(|f| crate::model::optimized::display::FontDef {
                family: f.font_family.clone(),
                size: f.font_size.as_deref().and_then(|s| s.parse().ok()),
                bold: f.font_weight.as_deref() == Some("bold"),
                italic: f.font_style.as_deref() == Some("italic"),
            });

    let word_font =
        defaults
            .word_font
            .as_ref()
            .map(|f| crate::model::optimized::display::FontDef {
                family: f.font_family.clone(),
                size: f.font_size.as_deref().and_then(|s| s.parse().ok()),
                bold: f.font_weight.as_deref() == Some("bold"),
                italic: f.font_style.as_deref() == Some("italic"),
            });

    let lyric_font =
        defaults
            .lyric_fonts
            .first()
            .map(|f| crate::model::optimized::display::FontDef {
                family: f.font.font_family.clone(),
                size: f.font.font_size.as_deref().and_then(|s| s.parse().ok()),
                bold: f.font.font_weight.as_deref() == Some("bold"),
                italic: f.font.font_style.as_deref() == Some("italic"),
            });

    let lyric_language = defaults
        .lyric_languages
        .first()
        .and_then(|ll| ll.lang.clone());

    Some(ScoreDefaults {
        page_layout,
        scaling,
        music_font,
        word_font,
        lyric_font,
        lyric_language,
        appearance: None,
    })
}
