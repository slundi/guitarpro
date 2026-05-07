use crate::model::beat::Beat;
use crate::model::effects::{FORTE, GP_BEND_SEMITONE, MIN_VELOCITY, VELOCITY_INCREMENT};
use crate::model::enums::*;
use crate::model::note::Note;
use crate::model::song::Song;

pub trait SongGpifExportOps {
    /// Serialize this `Song` to a GPIF XML string (GP6 flavour).
    fn write_gpif_xml(&self) -> String;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn velocity_to_dynamic(velocity: i16) -> &'static str {
    let levels: [(i16, &str); 8] = [
        (MIN_VELOCITY, "PPP"),
        (MIN_VELOCITY + VELOCITY_INCREMENT, "PP"),
        (MIN_VELOCITY + VELOCITY_INCREMENT * 2, "P"),
        (MIN_VELOCITY + VELOCITY_INCREMENT * 3, "MP"),
        (MIN_VELOCITY + VELOCITY_INCREMENT * 4, "MF"),
        (FORTE, "F"),
        (MIN_VELOCITY + VELOCITY_INCREMENT * 6, "FF"),
        (MIN_VELOCITY + VELOCITY_INCREMENT * 7, "FFF"),
    ];
    let mut best = "F";
    let mut best_diff = i16::MAX;
    for (v, name) in &levels {
        let diff = (velocity - v).abs();
        if diff < best_diff {
            best_diff = diff;
            best = name;
        }
    }
    best
}

fn note_value_str(value: u16) -> &'static str {
    match value {
        1 => "Whole",
        2 => "Half",
        4 => "Quarter",
        8 => "Eighth",
        16 => "16th",
        32 => "32nd",
        64 => "64th",
        128 => "128th",
        _ => "Quarter",
    }
}

fn slide_types_to_flags(slides: &[SlideType]) -> i32 {
    let mut flags = 0i32;
    for s in slides {
        flags |= match s {
            SlideType::ShiftSlideTo => 0x01,
            SlideType::LegatoSlideTo => 0x02,
            SlideType::OutDownwards => 0x04,
            SlideType::OutUpWards => 0x08,
            SlideType::IntoFromBelow => 0x10,
            SlideType::IntoFromAbove => 0x20,
            SlideType::None => 0,
        };
    }
    flags
}

fn harmonic_type_str(kind: &HarmonicType) -> &'static str {
    match kind {
        HarmonicType::Natural => "Natural",
        HarmonicType::Artificial => "Artificial",
        HarmonicType::Pinch => "Pinch",
        HarmonicType::Tapped => "Tap",
        HarmonicType::Semi => "Semi",
    }
}

fn direction_sign_str(d: &DirectionSign) -> &'static str {
    match d {
        DirectionSign::Coda => "Coda",
        DirectionSign::DoubleCoda => "DoubleCoda",
        DirectionSign::Segno => "Segno",
        DirectionSign::SegnoSegno => "SegnoSegno",
        DirectionSign::Fine => "Fine",
        DirectionSign::DaCapo => "DaCapo",
        DirectionSign::DaCapoAlCoda => "DaCapoAlCoda",
        DirectionSign::DaCapoAlDoubleCoda => "DaCapoAlDoubleCoda",
        DirectionSign::DaCapoAlFine => "DaCapoAlFine",
        DirectionSign::DaSegno => "DaSegno",
        DirectionSign::DaSegnoAlCoda => "DaSegnoAlCoda",
        DirectionSign::DaSegnoAlDoubleCoda => "DaSegnoAlDoubleCoda",
        DirectionSign::DaSegnoAlFine => "DaSegnoAlFine",
        DirectionSign::DaSegnoSegno => "DaSegnoSegno",
        DirectionSign::DaSegnoSegnoAlCoda => "DaSegnoSegnoAlCoda",
        DirectionSign::DaSegnoSegnoAlDoubleCoda => "DaSegnoSegnoAlDoubleCoda",
        DirectionSign::DaSegnoSegnoAlFine => "DaSegnoSegnoAlFine",
        DirectionSign::DaCoda => "DaCoda",
        DirectionSign::DaDoubleCoda => "DaDoubleCoda",
    }
}

// ---------------------------------------------------------------------------
// Note export
// ---------------------------------------------------------------------------

fn build_note_xml(note: &Note, note_id: i32) -> String {
    let mut s = format!("<Note id=\"{note_id}\"><Properties>");

    s.push_str(&format!(
        "<Property name=\"Fret\"><Fret>{}</Fret></Property>",
        note.value
    ));
    s.push_str(&format!(
        "<Property name=\"String\"><String>{}</String></Property>",
        note.string
    ));

    if note.effect.palm_mute {
        s.push_str("<Property name=\"PalmMuted\"><Enable /></Property>");
    }

    if let Some(bend) = &note.effect.bend {
        if bend.points.len() >= 3 {
            let origin = bend.points[0].value as f64 * GP_BEND_SEMITONE as f64;
            let dest = bend.points[2].value as f64 * GP_BEND_SEMITONE as f64;
            s.push_str(&format!(
                "<Property name=\"BendOriginValue\"><Float>{origin}</Float></Property>"
            ));
            s.push_str(&format!(
                "<Property name=\"BendDestinationValue\"><Float>{dest}</Float></Property>"
            ));
        }
    }

    if !note.effect.slides.is_empty() {
        let flags = slide_types_to_flags(&note.effect.slides);
        s.push_str(&format!(
            "<Property name=\"Slide\"><Flags>{flags}</Flags></Property>"
        ));
    }

    if let Some(harm) = &note.effect.harmonic {
        s.push_str(&format!(
            "<Property name=\"HarmonicType\"><HType>{}</HType></Property>",
            harmonic_type_str(&harm.kind)
        ));
        if let Some(fret) = harm.fret {
            s.push_str(&format!(
                "<Property name=\"HarmonicFret\"><HFret>{fret}</HFret></Property>"
            ));
        }
    }

    if note.effect.hammer {
        s.push_str("<Property name=\"HopoOrigin\"><Enable /></Property>");
    }

    if note.kind == NoteType::Dead {
        s.push_str("<Property name=\"Dead\"><Enable /></Property>");
    }

    s.push_str("</Properties>");

    if note.kind == NoteType::Tie {
        s.push_str("<Tie destination=\"true\" />");
    }
    if note.effect.vibrato {
        s.push_str("<Vibrato>Slight</Vibrato>");
    }
    if note.effect.let_ring {
        s.push_str("<LetRing />");
    }
    if note.effect.ghost_note {
        s.push_str("<AntiAccent>Normal</AntiAccent>");
    }

    let mut accent = 0i32;
    if note.effect.staccato {
        accent |= 0x01;
    }
    if note.effect.accentuated_note {
        accent |= 0x02;
    }
    if note.effect.heavy_accentuated_note {
        accent |= 0x04;
    }
    if accent != 0 {
        s.push_str(&format!("<Accent>{accent}</Accent>"));
    }

    if let Some(trill) = &note.effect.trill {
        s.push_str(&format!("<Trill>{}</Trill>", trill.fret));
    }
    if let Some(orn) = &note.effect.ornament {
        s.push_str(&format!("<Ornament>{}</Ornament>", escape_xml(orn)));
    }

    s.push_str("</Note>");
    s
}

// ---------------------------------------------------------------------------
// Beat export
// ---------------------------------------------------------------------------

struct BeatExport {
    beat_xml: String,
    rhythm_xml: String,
    note_xmls: Vec<String>,
}

fn build_beat_xml(
    beat: &Beat,
    beat_id: i32,
    rhythm_id: i32,
    next_note_id: &mut i32,
    prev_velocity: &mut i16,
) -> BeatExport {
    let mut note_xmls: Vec<String> = Vec::new();
    let mut note_ids: Vec<i32> = Vec::new();

    let is_grace_beat =
        !beat.notes.is_empty() && beat.notes.iter().all(|n| n.effect.grace.is_some());

    // Build note XMLs
    if beat.status == BeatStatus::Normal {
        for note in &beat.notes {
            let note_id = *next_note_id;
            *next_note_id += 1;
            note_ids.push(note_id);
            note_xmls.push(build_note_xml(note, note_id));
        }
    }

    // Determine beat velocity (from first note)
    let beat_velocity = beat
        .notes
        .first()
        .map(|n| n.velocity)
        .unwrap_or(*prev_velocity);

    let mut bxml = format!("<Beat id=\"{beat_id}\">");

    // Grace notes marker
    if is_grace_beat {
        let on_beat = beat.notes[0]
            .effect
            .grace
            .as_ref()
            .map(|g| g.is_on_beat)
            .unwrap_or(false);
        if on_beat {
            bxml.push_str("<GraceNotes>OnBeat</GraceNotes>");
        } else {
            bxml.push_str("<GraceNotes>BeforeBeat</GraceNotes>");
        }
    }

    // Notes reference
    if !note_ids.is_empty() {
        let ids_str: Vec<String> = note_ids.iter().map(|id| id.to_string()).collect();
        bxml.push_str(&format!("<Notes>{}</Notes>", ids_str.join(" ")));
    }

    // Dynamic (only when velocity changes)
    if beat_velocity != *prev_velocity {
        bxml.push_str(&format!(
            "<Dynamic>{}</Dynamic>",
            velocity_to_dynamic(beat_velocity)
        ));
        *prev_velocity = beat_velocity;
    }

    // Free text
    if !beat.text.is_empty() {
        bxml.push_str(&format!("<FreeText>{}</FreeText>", escape_xml(&beat.text)));
    }

    // Fade in
    if beat.effect.fade_in {
        bxml.push_str("<Fadding>FadeIn</Fadding>");
    }

    // Tremolo bar
    if let Some(trem) = &beat.effect.tremolo_bar {
        if trem.points.len() >= 3 {
            let dest = trem.points[2].value as f64 * GP_BEND_SEMITONE as f64;
            bxml.push_str(&format!("<Tremolo>{dest}</Tremolo>"));
        }
    }

    // Beat properties (Brush, Rasgueado, PickStroke)
    let has_brush = !matches!(beat.effect.stroke.direction, BeatStrokeDirection::None);
    let has_pick = !matches!(beat.effect.pick_stroke, BeatStrokeDirection::None);
    let has_rasg = beat.effect.has_rasgueado;

    if has_brush || has_pick || has_rasg {
        bxml.push_str("<Properties>");
        if has_brush {
            let dir = if matches!(beat.effect.stroke.direction, BeatStrokeDirection::Down) {
                "Down"
            } else {
                "Up"
            };
            bxml.push_str(&format!(
                "<Property name=\"Brush\"><Direction>{dir}</Direction></Property>"
            ));
        }
        if has_pick {
            let dir = if matches!(beat.effect.pick_stroke, BeatStrokeDirection::Down) {
                "Down"
            } else {
                "Up"
            };
            bxml.push_str(&format!(
                "<Property name=\"PickStroke\"><Direction>{dir}</Direction></Property>"
            ));
        }
        if has_rasg {
            bxml.push_str("<Property name=\"Rasgueado\"><Enable /></Property>");
        }
        bxml.push_str("</Properties>");
    }

    // Rhythm reference
    bxml.push_str(&format!("<Rhythm ref=\"{rhythm_id}\"/>"));

    bxml.push_str("</Beat>");

    // Rhythm XML
    let dur = &beat.duration;
    let mut rxml = format!(
        "<Rhythm id=\"{rhythm_id}\"><NoteValue>{}</NoteValue>",
        note_value_str(dur.value)
    );
    if dur.dotted {
        rxml.push_str("<AugmentationDot count=\"1\"/>");
    } else if dur.double_dotted {
        rxml.push_str("<AugmentationDot count=\"2\"/>");
    }
    if dur.tuplet_enters != 1 || dur.tuplet_times != 1 {
        rxml.push_str(&format!(
            "<PrimaryTuplet num=\"{}\" den=\"{}\"/>",
            dur.tuplet_enters, dur.tuplet_times
        ));
    }
    rxml.push_str("</Rhythm>");

    BeatExport {
        beat_xml: bxml,
        rhythm_xml: rxml,
        note_xmls,
    }
}

// ---------------------------------------------------------------------------
// Main implementation
// ---------------------------------------------------------------------------

impl SongGpifExportOps for Song {
    fn write_gpif_xml(&self) -> String {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?><GPIF>");

        // Score
        xml.push_str("<Score>");
        xml.push_str(&format!("<Title>{}</Title>", escape_xml(&self.name)));
        xml.push_str(&format!(
            "<SubTitle>{}</SubTitle>",
            escape_xml(&self.subtitle)
        ));
        xml.push_str(&format!("<Artist>{}</Artist>", escape_xml(&self.artist)));
        xml.push_str(&format!("<Album>{}</Album>", escape_xml(&self.album)));
        xml.push_str(&format!("<Words>{}</Words>", escape_xml(&self.words)));
        xml.push_str(&format!("<Music>{}</Music>", escape_xml(&self.author)));
        xml.push_str(&format!(
            "<Copyright>{}</Copyright>",
            escape_xml(&self.copyright)
        ));
        xml.push_str(&format!(
            "<Tabber>{}</Tabber>",
            escape_xml(&self.transcriber)
        ));
        xml.push_str(&format!(
            "<Instructions>{}</Instructions>",
            escape_xml(&self.comments)
        ));
        let notices = self.notice.join("\n");
        xml.push_str(&format!("<Notices>{}</Notices>", escape_xml(&notices)));
        xml.push_str("</Score>");

        // MasterTrack
        let num_tracks = self.tracks.len();
        let track_ids: Vec<String> = (0..num_tracks).map(|i| i.to_string()).collect();
        xml.push_str("<MasterTrack>");
        xml.push_str(&format!("<Tracks>{}</Tracks>", track_ids.join(" ")));
        xml.push_str("<Automations>");
        // Bar-0 tempo (always)
        xml.push_str(&format!(
            "<Automation><Type>Tempo</Type><Value>{} 2</Value><Bar>0</Bar><Position>0</Position></Automation>",
            self.tempo
        ));
        // Per-measure tempo changes
        for (i, mh) in self.measure_headers.iter().enumerate() {
            if i == 0 {
                continue;
            }
            if mh.tempo > 0 {
                xml.push_str(&format!(
                    "<Automation><Type>Tempo</Type><Value>{} 2</Value><Bar>{i}</Bar><Position>0</Position></Automation>",
                    mh.tempo
                ));
            }
        }
        xml.push_str("</Automations>");
        xml.push_str("</MasterTrack>");

        // Tracks
        xml.push_str("<Tracks>");
        for (t_idx, track) in self.tracks.iter().enumerate() {
            xml.push_str(&format!("<Track id=\"{t_idx}\">"));
            xml.push_str(&format!("<Name>{}</Name>", escape_xml(&track.name)));
            xml.push_str(&format!(
                "<ShortName>{}</ShortName>",
                escape_xml(&track.short_name)
            ));
            let r = (track.color >> 16) & 0xFF;
            let g = (track.color >> 8) & 0xFF;
            let b = track.color & 0xFF;
            xml.push_str(&format!("<Color>{r} {g} {b}</Color>"));
            xml.push_str("<Properties>");
            let pitches: Vec<String> = track.strings.iter().map(|(_, p)| p.to_string()).collect();
            xml.push_str(&format!(
                "<Property name=\"Tuning\"><Pitches>{}</Pitches></Property>",
                pitches.join(" ")
            ));
            xml.push_str("</Properties>");
            xml.push_str("<GeneralMidi>");
            if let Some(prog) = track.midi_program_gpif {
                xml.push_str(&format!("<Program>{prog}</Program>"));
            }
            xml.push_str(&format!("<Port>{}</Port>", track.port));
            xml.push_str(&format!(
                "<PrimaryChannel>{}</PrimaryChannel>",
                track.channel_index
            ));
            xml.push_str(&format!(
                "<SecondaryChannel>{}</SecondaryChannel>",
                (track.channel_index + 1).min(15)
            ));
            xml.push_str("</GeneralMidi>");
            if track.transpose_chromatic != 0 || track.transpose_octave != 0 {
                xml.push_str("<Transpose>");
                xml.push_str(&format!(
                    "<Chromatic>{}</Chromatic>",
                    track.transpose_chromatic
                ));
                xml.push_str(&format!("<Octave>{}</Octave>", track.transpose_octave));
                xml.push_str("</Transpose>");
            }
            xml.push_str("</Track>");
        }
        xml.push_str("</Tracks>");

        // Build all bars/voices/beats/notes/rhythms with sequential IDs
        let num_measures = self.measure_headers.len();

        // (bar_id, voice_ids_string)
        let mut bars: Vec<(i32, String)> = Vec::new();
        let mut voices_xml: Vec<String> = Vec::new();
        let mut beats_xml: Vec<String> = Vec::new();
        let mut notes_xml: Vec<String> = Vec::new();
        let mut rhythms_xml: Vec<String> = Vec::new();

        let mut next_voice_id: i32 = 0;
        let mut next_beat_id: i32 = 0;
        let mut next_note_id: i32 = 0;
        let mut next_rhythm_id: i32 = 0;

        // bar_id = t_idx * num_measures + m_idx
        for t_idx in 0..self.tracks.len() {
            let track = &self.tracks[t_idx];
            // Velocity persists across all measures/voices for a given track
            let mut prev_velocity: i16 = FORTE;

            for m_idx in 0..num_measures {
                let bar_id = (t_idx * num_measures + m_idx) as i32;
                let measure = &track.measures[m_idx];
                let mut voice_ids: Vec<String> = Vec::new();

                for voice in &measure.voices {
                    if voice.beats.is_empty() {
                        voice_ids.push("-1".to_string());
                        continue;
                    }

                    let voice_id = next_voice_id;
                    next_voice_id += 1;
                    let mut beat_ids: Vec<i32> = Vec::new();

                    for beat in &voice.beats {
                        let beat_id = next_beat_id;
                        next_beat_id += 1;
                        let rhythm_id = next_rhythm_id;
                        next_rhythm_id += 1;
                        beat_ids.push(beat_id);

                        let export = build_beat_xml(
                            beat,
                            beat_id,
                            rhythm_id,
                            &mut next_note_id,
                            &mut prev_velocity,
                        );
                        beats_xml.push(export.beat_xml);
                        rhythms_xml.push(export.rhythm_xml);
                        notes_xml.extend(export.note_xmls);
                    }

                    let beat_ids_str: Vec<String> =
                        beat_ids.iter().map(|id| id.to_string()).collect();
                    voices_xml.push(format!(
                        "<Voice id=\"{voice_id}\"><Beats>{}</Beats></Voice>",
                        beat_ids_str.join(" ")
                    ));
                    voice_ids.push(voice_id.to_string());
                }

                if let Some(sm) = &measure.simile_mark {
                    bars.push((
                        bar_id,
                        format!("voices=\"{}\" simile=\"{}\"", voice_ids.join(" "), sm),
                    ));
                } else {
                    bars.push((bar_id, voice_ids.join(" ")));
                }
            }
        }

        // MasterBars
        xml.push_str("<MasterBars>");
        for (m_idx, mh) in self.measure_headers.iter().enumerate() {
            xml.push_str("<MasterBar>");
            xml.push_str(&format!(
                "<Time>{}/{}</Time>",
                mh.time_signature.numerator, mh.time_signature.denominator.value
            ));
            let mode = if mh.key_signature.is_minor {
                "Minor"
            } else {
                "Major"
            };
            xml.push_str(&format!(
                "<Key><AccidentalCount>{}</AccidentalCount><Mode>{mode}</Mode></Key>",
                mh.key_signature.key
            ));
            let bar_ids: Vec<String> = (0..self.tracks.len())
                .map(|t| (t * num_measures + m_idx).to_string())
                .collect();
            xml.push_str(&format!("<Bars>{}</Bars>", bar_ids.join(" ")));

            if mh.repeat_open || mh.repeat_close > 0 {
                let start = if mh.repeat_open { "true" } else { "false" };
                let end = if mh.repeat_close > 0 { "true" } else { "false" };
                xml.push_str(&format!(
                    "<Repeat start=\"{start}\" end=\"{end}\" count=\"{}\"/>",
                    mh.repeat_close
                ));
            }
            if mh.repeat_alternative > 0 {
                let mut endings: Vec<String> = Vec::new();
                for i in 0..8u8 {
                    if (mh.repeat_alternative >> i) & 1 != 0 {
                        endings.push((i + 1).to_string());
                    }
                }
                xml.push_str(&format!(
                    "<AlternateEndings>{}</AlternateEndings>",
                    endings.join(" ")
                ));
            }
            if mh.double_bar {
                xml.push_str("<DoubleBar/>");
            }
            if let Some(marker) = &mh.marker {
                xml.push_str(&format!(
                    "<Section><Text>{}</Text></Section>",
                    escape_xml(&marker.title)
                ));
            }
            if !mh.fermatas.is_empty() {
                xml.push_str("<Fermatas>");
                for (ftype, offset) in &mh.fermatas {
                    xml.push_str(&format!(
                        "<Fermata><Type>{ftype}</Type><Offset>{offset}</Offset></Fermata>"
                    ));
                }
                xml.push_str("</Fermatas>");
            }
            if mh.free_time {
                xml.push_str("<FreeTime/>");
            }
            if let Some(dir) = &mh.direction {
                xml.push_str(&format!(
                    "<Directions><Jump>{}</Jump></Directions>",
                    direction_sign_str(dir)
                ));
            }
            xml.push_str("</MasterBar>");
        }
        xml.push_str("</MasterBars>");

        // Bars
        xml.push_str("<Bars>");
        for (bar_id, meta) in &bars {
            // meta is either "voice_ids_string" or "voices=... simile=..." (for simile marks)
            if meta.starts_with("voices=") {
                // parse out voices and simile
                // format: voices="v1 v2" simile="X"
                if let (Some(vi_start), Some(vi_end), Some(sm_start)) = (
                    meta.find("voices=\"").map(|p| p + 8),
                    meta.find("\" simile=\""),
                    meta.find("simile=\"").map(|p| p + 8),
                ) {
                    let voices_str = &meta[vi_start..vi_end];
                    let simile_end = meta[sm_start..].find('"').map(|p| sm_start + p);
                    if let Some(se) = simile_end {
                        let simile_val = &meta[sm_start..se];
                        xml.push_str(&format!(
                            "<Bar id=\"{bar_id}\"><Voices>{voices_str}</Voices><SimileMark>{simile_val}</SimileMark></Bar>"
                        ));
                    } else {
                        xml.push_str(&format!(
                            "<Bar id=\"{bar_id}\"><Voices>{voices_str}</Voices></Bar>"
                        ));
                    }
                } else {
                    xml.push_str(&format!(
                        "<Bar id=\"{bar_id}\"><Voices>{meta}</Voices></Bar>"
                    ));
                }
            } else {
                xml.push_str(&format!(
                    "<Bar id=\"{bar_id}\"><Voices>{meta}</Voices></Bar>"
                ));
            }
        }
        xml.push_str("</Bars>");

        // Voices
        xml.push_str("<Voices>");
        for v in &voices_xml {
            xml.push_str(v);
        }
        xml.push_str("</Voices>");

        // Beats
        xml.push_str("<Beats>");
        for b in &beats_xml {
            xml.push_str(b);
        }
        xml.push_str("</Beats>");

        // Notes
        xml.push_str("<Notes>");
        for n in &notes_xml {
            xml.push_str(n);
        }
        xml.push_str("</Notes>");

        // Rhythms
        xml.push_str("<Rhythms>");
        for r in &rhythms_xml {
            xml.push_str(r);
        }
        xml.push_str("</Rhythms>");

        xml.push_str("</GPIF>");
        xml
    }
}
