//! MSCZ notation tests: dynamics, hairpins, articulations and ornaments,
//! fermatas, breath marks, grace notes, chord symbols, voltas, barline
//! styles and jumps/markers, MSCX ↔ LoadedScore.

use crate::convert::mscz::notation::{
    ChordMark, barline_style, barline_to_mscx, chord_marks_from_symbol, dynamic_from_mscx,
    dynamic_to_mscx, jump_kind, jump_kind_to_mscx,
};
use crate::convert::mscz::{loaded_score_to_mscx, mscx_to_loaded_score};
use crate::error::{GpError, GpResult};
use crate::io::mscz::parse_mscx;
use crate::model::optimized::{
    LoadedScore,
    beat::{Beat, Dynamic},
    effect::{BeatEvent, HairpinKind},
    note::{Articulation, NoteValue, Ornament, PitchStep, TieType},
    timeline::{BarlineStyle, EndingKind, JumpKind},
};

/// Three measures: a grace note before a chord carrying marks, with the
/// segment elements MuseScore writes before them; a hairpin across the
/// barline; a two-staff barline style difference; a volta over bar 2; a
/// segno and a D.S. al Fine.
const MSCX_NOTATION: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<museScore version="4.20">
  <Score>
    <Division>480</Division>
    <metaTag name="workTitle">Notation</metaTag>
    <Part id="1">
      <Staff id="1"><StaffType group="pitched"><name>stdNormal</name></StaffType></Staff>
      <trackName>Piano</trackName>
      <Instrument id="piano"><longName>Piano</longName></Instrument>
    </Part>
    <Part id="2">
      <Staff id="2"><StaffType group="pitched"><name>stdNormal</name></StaffType></Staff>
      <trackName>Flute</trackName>
      <Instrument id="flute"><longName>Flute</longName></Instrument>
    </Part>
    <Staff id="1">
      <Measure>
        <Marker><style>repeat_left</style><text><sym>segno</sym></text><label>segno</label></Marker>
        <voice>
          <TimeSig><sigN>4</sigN><sigD>4</sigD></TimeSig>
          <Harmony><root>14</root><name>m7</name><base>11</base></Harmony>
          <Dynamic><subtype>pp</subtype><velocity>33</velocity></Dynamic>
          <Spanner type="HairPin">
            <HairPin><subtype>0</subtype></HairPin>
            <next><location><measures>1</measures></location></next>
          </Spanner>
          <Chord><acciaccatura/><durationType>eighth</durationType><Note><pitch>62</pitch><tpc>16</tpc></Note></Chord>
          <Chord>
            <durationType>half</durationType>
            <Articulation><subtype>articAccentStaccatoBelow</subtype></Articulation>
            <Articulation><subtype>ornamentMordentInverted</subtype></Articulation>
            <ChordLine><subtype>2</subtype></ChordLine>
            <Note><pitch>60</pitch><tpc>14</tpc></Note>
          </Chord>
          <Breath><symbol>caesura</symbol></Breath>
          <Fermata><subtype>fermataAbove</subtype></Fermata>
          <Dynamic><subtype>other-dynamics</subtype><text>sempre <i>p</i></text></Dynamic>
          <Rest><durationType>half</durationType></Rest>
          <BarLine><subtype>double</subtype></BarLine>
        </voice>
      </Measure>
      <Measure>
        <voice>
          <Spanner type="HairPin"><prev><location><measures>-1</measures></location></prev></Spanner>
          <Spanner type="Volta">
            <Volta><endHookType>1</endHookType><beginText>1.</beginText><endings>1, 2</endings></Volta>
            <next><location><measures>1</measures></location></next>
          </Spanner>
          <Harmony><name>N.C.</name></Harmony>
          <Chord><durationType>whole</durationType><Ornament><subtype>ornamentTurn</subtype></Ornament><Tremolo><subtype>r16</subtype></Tremolo><Note><pitch>64</pitch><tpc>18</tpc></Note></Chord>
        </voice>
      </Measure>
      <Measure>
        <Jump><text>D.S. al Fine</text><jumpTo>segno</jumpTo><playUntil>fine</playUntil><continueAt></continueAt></Jump>
        <voice>
          <Spanner type="Volta"><prev><location><measures>-1</measures></location></prev></Spanner>
          <Rest><durationType>measure</durationType></Rest>
        </voice>
      </Measure>
    </Staff>
    <Staff id="2">
      <Measure><voice><Rest><durationType>measure</durationType></Rest></voice></Measure>
      <Measure><voice><Rest><durationType>measure</durationType></Rest><BarLine><subtype>end</subtype></BarLine></voice></Measure>
      <Measure><voice><Rest><durationType>measure</durationType></Rest></voice></Measure>
    </Staff>
  </Score>
</museScore>
"#;

fn present<T>(value: Option<T>, field: &'static str) -> GpResult<T> {
    value.ok_or(GpError::MissingState { field })
}

fn beats(score: &LoadedScore, track: usize, measure: usize) -> GpResult<&[Beat]> {
    let data = present(
        score.score.tracks[track].measures.values().nth(measure),
        "measure",
    )?;
    Ok(&present(data.voices.get(&0), "voice")?.beats)
}

fn hairpins(beat: &Beat) -> Vec<(u8, HairpinKind)> {
    beat.events
        .iter()
        .filter_map(|event| match event {
            BeatEvent::Hairpin(hairpin) => Some((hairpin.id, hairpin.kind)),
            _ => None,
        })
        .collect()
}

/// Everything [`MSCX_NOTATION`] marks, checked on a converted score.
fn check_notation(score: &LoadedScore) -> GpResult<()> {
    let timeline = &score.score.timeline;
    let jumps: Vec<Vec<JumpKind>> = timeline
        .iter()
        .map(|m| m.navigation.iter().map(|e| e.kind).collect())
        .collect();
    assert_eq!(
        jumps,
        vec![
            vec![JumpKind::Segno],
            vec![],
            vec![JumpKind::DalSegnoAlFine]
        ]
    );

    // The barline styles: the timeline's from the first staff, the second
    // staff's own where it differs.
    let style = |m: usize| timeline[m].barline_right.as_ref().map(|b| b.style);
    assert_eq!(style(0), Some(BarlineStyle::LightLight));
    let flute = |m: usize| {
        score.score.tracks[1]
            .measures
            .values()
            .nth(m)
            .and_then(|data| data.barline_right.as_ref())
            .map(|b| b.style)
    };
    assert_eq!(flute(0), Some(BarlineStyle::Regular));
    assert_eq!(flute(1), Some(BarlineStyle::LightHeavy));

    // The volta: over bar 2 only (its end element opens bar 3), closed.
    let start = present(
        timeline[1]
            .barline_left
            .as_ref()
            .and_then(|b| b.ending.as_ref()),
        "volta start",
    )?;
    assert_eq!(start.kind, EndingKind::Start);
    assert_eq!(start.numbers, vec![1, 2]);
    let end = present(
        timeline[1]
            .barline_right
            .as_ref()
            .and_then(|b| b.ending.as_ref()),
        "volta end",
    )?;
    assert_eq!(end.kind, EndingKind::Stop);

    // The grace note and segment elements go with the chord after them.
    let bar1 = beats(score, 0, 0)?;
    assert_eq!(bar1.len(), 2);
    let main = &bar1[0];
    assert_eq!(main.duration.base, NoteValue::Half);
    let grace = present(main.grace_notes.first(), "grace note")?;
    assert!(grace.slash && !grace.after);
    assert_eq!(grace.duration.map(|d| d.base), Some(NoteValue::Eighth));
    assert_eq!(main.dynamic, Some(Dynamic::PP));
    let chord = present(main.chord.as_ref(), "chord symbol")?;
    assert_eq!(chord.root.map(|r| r.step), Some(PitchStep::C));
    assert_eq!(chord.kind, "m7");
    assert_eq!(
        chord.bass.map(|b| (b.step, b.alter)),
        Some((PitchStep::E, -1))
    );
    let note = &main.notes[0];
    assert_eq!(
        note.articulations,
        vec![
            Articulation::Accent,
            Articulation::Staccato,
            Articulation::Doit,
            Articulation::Caesura
        ]
    );
    assert_eq!(
        note.ornaments,
        vec![Ornament::InvertedMordent { long: false }]
    );

    let rest = &bar1[1];
    assert!(rest.fermata);
    assert_eq!(rest.dynamic, Some(Dynamic::Other("sempre p".into())));
    // The hairpin runs from the chord to the rest (its end element opens
    // bar 2).
    let start = hairpins(main);
    assert_eq!(start.len(), 1);
    assert_eq!(start[0].1, HairpinKind::CrescendoStart);
    assert_eq!(hairpins(rest), vec![(start[0].0, HairpinKind::Stop)]);

    let bar2 = beats(score, 0, 1)?;
    let turn = &bar2[0];
    assert_eq!(
        turn.notes[0].ornaments,
        vec![Ornament::Turn, Ornament::Tremolo(2)]
    );
    let no_chord = present(turn.chord.as_ref(), "N.C.")?;
    assert!(no_chord.root.is_none());
    assert!(hairpins(turn).is_empty());
    Ok(())
}

#[test]
fn reads_notation() -> GpResult<()> {
    let outcome = mscx_to_loaded_score(&parse_mscx(MSCX_NOTATION)?);
    assert!(outcome.report.is_empty(), "{:?}", outcome.report);
    check_notation(&outcome.score)
}

#[test]
fn writes_notation_back() -> GpResult<()> {
    let score = mscx_to_loaded_score(&parse_mscx(MSCX_NOTATION)?).score;
    let written = loaded_score_to_mscx(&score);
    let reread = mscx_to_loaded_score(&parse_mscx(&written.raw_xml)?);
    assert!(reread.report.is_empty(), "{:?}", reread.report);
    check_notation(&reread.score)
}

/// A volta end pointing where no volta starts is still paired, with the
/// last one opened, but reported.
#[test]
fn a_volta_end_pointing_nowhere_is_reported() -> GpResult<()> {
    let end = r#"<Spanner type="Volta"><prev><location><measures>-1</measures></location></prev></Spanner>"#;
    assert!(MSCX_NOTATION.contains(end));
    let xml = MSCX_NOTATION.replace(
        end,
        r#"<Spanner type="Volta"><prev><location><measures>-2</measures></location></prev></Spanner>"#,
    );
    let outcome = mscx_to_loaded_score(&parse_mscx(&xml)?);
    assert_eq!(outcome.report.get("Volta (end not at its start)"), 1);
    let clean = mscx_to_loaded_score(&parse_mscx(MSCX_NOTATION)?);
    assert_eq!(clean.report.get("Volta (end not at its start)"), 0);
    Ok(())
}

#[test]
fn hairpin_ends_pair_with_the_oldest_open_one() -> GpResult<()> {
    // A diminuendo starting where a crescendo ends.
    let xml = MSCX_NOTATION.replace(
        r#"<Fermata><subtype>fermataAbove</subtype></Fermata>"#,
        r#"<Fermata><subtype>fermataAbove</subtype></Fermata>
          <Spanner type="HairPin"><HairPin><subtype>1</subtype></HairPin><next><location><measures>1</measures></location></next></Spanner>"#,
    );
    let xml = xml.replace(
        r#"<Spanner type="Volta">
            <Volta>"#,
        r#"<Spanner type="HairPin"><prev><location><measures>-1</measures></location></prev></Spanner>
          <Spanner type="Volta">
            <Volta>"#,
    );
    let score = mscx_to_loaded_score(&parse_mscx(&xml)?).score;
    let bar1 = beats(&score, 0, 0)?;
    let crescendo = hairpins(&bar1[0])[0].0;
    let rest = hairpins(&bar1[1]);
    let diminuendo = present(
        rest.iter()
            .find(|(_, kind)| *kind == HairpinKind::DecrescendoStart),
        "diminuendo",
    )?
    .0;
    assert_ne!(crescendo, diminuendo);
    let stops: Vec<u8> = rest
        .iter()
        .filter(|(_, kind)| *kind == HairpinKind::Stop)
        .map(|(id, _)| *id)
        .collect();
    assert_eq!(stops, vec![crescendo, diminuendo]);
    Ok(())
}

#[test]
fn dynamics_without_subtype_use_other_dynamics() {
    let written = dynamic_to_mscx(&Dynamic::SFZP);
    assert_eq!(written.subtype.as_deref(), Some("other-dynamics"));
    assert_eq!(dynamic_from_mscx(&written), Some(Dynamic::SFZP));
    let written = dynamic_to_mscx(&Dynamic::MF);
    assert_eq!(written.velocity, Some(80));
    assert_eq!(dynamic_from_mscx(&written), Some(Dynamic::MF));
}

#[test]
fn navigation_marks_round_trip() {
    for kind in [
        JumpKind::Segno,
        JumpKind::Coda,
        JumpKind::Fine,
        JumpKind::ToCoda,
        JumpKind::DaCapo,
        JumpKind::DaCapoAlFine,
        JumpKind::DaCapoAlCoda,
        JumpKind::DalSegno,
        JumpKind::DalSegnoAlFine,
        JumpKind::DalSegnoAlCoda,
    ] {
        assert_eq!(
            jump_kind_to_mscx(kind).as_ref().and_then(jump_kind),
            Some(kind)
        );
    }
}

#[test]
fn barline_styles_round_trip() {
    for style in [
        BarlineStyle::Dotted,
        BarlineStyle::Dashed,
        BarlineStyle::Heavy,
        BarlineStyle::LightLight,
        BarlineStyle::LightHeavy,
        BarlineStyle::HeavyLight,
        BarlineStyle::HeavyHeavy,
        BarlineStyle::Tick,
        BarlineStyle::Short,
        BarlineStyle::None,
    ] {
        assert_eq!(
            barline_to_mscx(style).as_ref().map(barline_style),
            Some(style)
        );
    }
    assert!(barline_to_mscx(BarlineStyle::Regular).is_none());
}

#[test]
fn combined_articulation_symbols_split() {
    let marks = chord_marks_from_symbol("articAccentStaccatoBelow");
    assert!(matches!(
        marks.as_slice(),
        [
            ChordMark::Articulation(Articulation::Accent),
            ChordMark::Articulation(Articulation::Staccato)
        ]
    ));
    assert!(matches!(
        chord_marks_from_symbol("ornamentMordentInverted").as_slice(),
        [ChordMark::Ornament(Ornament::InvertedMordent {
            long: false
        })]
    ));
    assert!(matches!(
        chord_marks_from_symbol("fermataAbove").as_slice(),
        [ChordMark::Fermata]
    ));
}

/// Hairpin ids wrap at 255 but skip those still open: a hairpin open
/// across 255 others keeps its id to itself.
#[test]
fn hairpin_ids_skip_open_hairpins_when_they_wrap() -> GpResult<()> {
    use crate::convert::mscz::{LossReport, to_optimized::number_hairpins};
    use crate::model::optimized::effect::HairpinEvent;

    let mut score = mscx_to_loaded_score(&parse_mscx(&with_second_voice(""))?).score;
    let measures = &mut score.score.tracks[0].measures;
    let hairpin = |kind| {
        BeatEvent::Hairpin(HairpinEvent {
            id: 0,
            kind,
            line: false,
        })
    };
    let bar1 = present(measures.values_mut().next(), "bar 1")?;
    for voice in bar1.voices.values_mut() {
        for beat in &mut voice.beats {
            beat.events
                .retain(|event| !matches!(event, BeatEvent::Hairpin(_)));
        }
    }
    // Voice 0 opens one on its first beat and closes it on its second.
    let first = present(bar1.voices.get_mut(&0), "voice 0")?;
    first.beats[0]
        .events
        .push(hairpin(HairpinKind::CrescendoStart));
    present(first.beats.get_mut(1), "second beat")?
        .events
        .push(hairpin(HairpinKind::Stop));
    // Voice 1 opens and closes 254 in between, then opens one more.
    let second = present(bar1.voices.get_mut(&1), "voice 1")?;
    let events = &mut second.beats[0].events;
    events.extend((0..254).map(|_| hairpin(HairpinKind::CrescendoStart)));
    events.extend((0..254).map(|_| hairpin(HairpinKind::Stop)));
    events.push(hairpin(HairpinKind::DecrescendoStart));
    for data in measures.values_mut().skip(1) {
        for voice in data.voices.values_mut() {
            for beat in &mut voice.beats {
                beat.events
                    .retain(|event| !matches!(event, BeatEvent::Hairpin(_)));
            }
        }
    }
    number_hairpins(measures, &mut LossReport::new());

    let bar1 = present(measures.values().next(), "bar 1")?;
    let first = &present(bar1.voices.get(&0), "voice 0")?.beats;
    let open = hairpins(&first[0])[0].0;
    assert_eq!(hairpins(&first[1]), vec![(open, HairpinKind::Stop)]);
    let last = present(
        hairpins(&present(bar1.voices.get(&1), "voice 1")?.beats[0])
            .last()
            .copied(),
        "last hairpin",
    )?;
    assert_eq!(last.1, HairpinKind::DecrescendoStart);
    assert_ne!(last.0, open);
    Ok(())
}

/// A hairpin starting and stopping on a beat without written length
/// still pairs.
#[test]
fn hairpins_on_a_beat_without_length_pair() -> GpResult<()> {
    use crate::model::optimized::effect::HairpinEvent;

    let mut score = mscx_to_loaded_score(&parse_mscx(MSCX_NOTATION)?).score;
    let measures = &mut score.score.tracks[0].measures;
    for data in measures.values_mut() {
        for voice in data.voices.values_mut() {
            for beat in &mut voice.beats {
                beat.events
                    .retain(|event| !matches!(event, BeatEvent::Hairpin(_)));
            }
        }
    }
    let bar2 = present(measures.values_mut().nth(1), "bar 2")?;
    let beat = &mut present(bar2.voices.get_mut(&0), "voice 0")?.beats[0];
    // Too short for a tick.
    beat.duration.base = NoteValue::Other(8192);
    beat.duration.dots = 0;
    for kind in [HairpinKind::CrescendoStart, HairpinKind::Stop] {
        beat.events.push(BeatEvent::Hairpin(HairpinEvent {
            id: 0,
            kind,
            line: false,
        }));
    }
    let xml = loaded_score_to_mscx(&score).raw_xml;
    assert_eq!(xml.matches(r#"<Spanner type="HairPin">"#).count(), 2);
    Ok(())
}

/// A "cresc." line stays a line, not a hairpin.
#[test]
fn cresc_lines_round_trip_as_lines() -> GpResult<()> {
    let xml = MSCX_NOTATION.replacen(
        "<HairPin><subtype>0</subtype></HairPin>",
        "<HairPin><subtype>2</subtype></HairPin>",
        1,
    );
    assert_ne!(xml, MSCX_NOTATION);
    let outcome = mscx_to_loaded_score(&parse_mscx(&xml)?);
    assert!(outcome.report.is_empty(), "{:?}", outcome.report);
    let written = loaded_score_to_mscx(&outcome.score).raw_xml;
    let subtypes: Vec<&str> = written
        .split("<HairPin>")
        .skip(1)
        .filter_map(|rest| rest.split("<subtype>").nth(1)?.split('<').next())
        .collect();
    assert_eq!(subtypes, vec!["2"], "{written}");
    Ok(())
}

#[test]
fn hairpins_starting_together_both_pair() -> GpResult<()> {
    // A "cresc." line starting on the same chord as the crescendo, both
    // ending at bar 2.
    let xml = MSCX_NOTATION.replacen(
        r#"<Spanner type="HairPin">
            <HairPin><subtype>0</subtype></HairPin>"#,
        r#"<Spanner type="HairPin"><HairPin><subtype>2</subtype></HairPin><next><location><measures>1</measures></location></next></Spanner>
          <Spanner type="HairPin">
            <HairPin><subtype>0</subtype></HairPin>"#,
        1,
    );
    let xml = xml.replacen(
        r#"<Spanner type="HairPin"><prev>"#,
        r#"<Spanner type="HairPin"><prev><location><measures>-1</measures></location></prev></Spanner>
          <Spanner type="HairPin"><prev>"#,
        1,
    );
    let outcome = mscx_to_loaded_score(&parse_mscx(&xml)?);
    assert!(outcome.report.is_empty(), "{:?}", outcome.report);
    let bar1 = beats(&outcome.score, 0, 0)?;
    let starts: Vec<u8> = hairpins(&bar1[0]).iter().map(|(id, _)| *id).collect();
    assert_eq!(starts.len(), 2);
    assert_ne!(starts[0], starts[1]);
    let stops: Vec<u8> = hairpins(&bar1[1]).iter().map(|(id, _)| *id).collect();
    assert_eq!(stops, starts);
    Ok(())
}

#[test]
fn hairpin_ends_after_a_leading_grace_chord_follow_the_measure_before() -> GpResult<()> {
    // Bar 2 opens with a grace chord before the crescendo's end element.
    let xml = MSCX_NOTATION.replacen(
        r#"<Spanner type="HairPin"><prev>"#,
        r#"<Chord><acciaccatura/><durationType>eighth</durationType><Note><pitch>62</pitch><tpc>16</tpc></Note></Chord>
          <Spanner type="HairPin"><prev>"#,
        1,
    );
    assert_ne!(xml, MSCX_NOTATION);
    let outcome = mscx_to_loaded_score(&parse_mscx(&xml)?);
    assert!(outcome.report.is_empty(), "{:?}", outcome.report);
    let bar1 = beats(&outcome.score, 0, 0)?;
    let crescendo = hairpins(&bar1[0])[0].0;
    assert_eq!(hairpins(&bar1[1]), vec![(crescendo, HairpinKind::Stop)]);
    Ok(())
}

#[test]
fn harmony_with_an_invalid_root_is_reported() -> GpResult<()> {
    let xml = MSCX_NOTATION.replacen("<root>14</root>", "<root>35</root>", 1);
    let outcome = mscx_to_loaded_score(&parse_mscx(&xml)?);
    assert_eq!(outcome.report.get("Harmony (invalid root)"), 1);
    assert!(beats(&outcome.score, 0, 0)?[0].chord.is_none());
    Ok(())
}

const GRACE_CHORD: &str = r#"<Chord><acciaccatura/><durationType>eighth</durationType><Note><pitch>62</pitch><tpc>16</tpc></Note></Chord>"#;

#[test]
fn grace_chords_before_a_rest_wait_for_the_next_chord() -> GpResult<()> {
    let xml = MSCX_NOTATION.replacen(
        GRACE_CHORD,
        &format!("{GRACE_CHORD}\n          <Rest><durationType>eighth</durationType></Rest>"),
        1,
    );
    assert_ne!(xml, MSCX_NOTATION);
    let outcome = mscx_to_loaded_score(&parse_mscx(&xml)?);
    assert!(outcome.report.is_empty(), "{:?}", outcome.report);
    let bar1 = beats(&outcome.score, 0, 0)?;
    assert!(bar1[0].notes.is_empty() && bar1[0].grace_notes.is_empty());
    assert_eq!(bar1[1].grace_notes.len(), 1);
    assert!(!bar1[1].grace_notes[0].after);
    // What was written before the grace chord waits with it.
    assert!(bar1[0].dynamic.is_none() && bar1[0].chord.is_none());
    assert!(hairpins(&bar1[0]).is_empty());
    assert_eq!(bar1[1].dynamic, Some(Dynamic::PP));
    assert!(bar1[1].chord.is_some());
    assert!(!hairpins(&bar1[1]).is_empty());
    Ok(())
}

#[test]
fn grace_chords_without_a_main_chord_are_reported() -> GpResult<()> {
    // A grace chord after the measure's last beat.
    let xml = MSCX_NOTATION.replacen(
        r#"<Rest><durationType>half</durationType></Rest>"#,
        &format!("<Rest><durationType>half</durationType></Rest>\n          {GRACE_CHORD}"),
        1,
    );
    assert_ne!(xml, MSCX_NOTATION);
    let outcome = mscx_to_loaded_score(&parse_mscx(&xml)?);
    assert_eq!(outcome.report.get("Grace (without a main chord)"), 1);
    let bar1 = beats(&outcome.score, 0, 0)?;
    assert!(
        bar1.iter()
            .all(|beat| beat.grace_notes.iter().all(|g| !g.after))
    );
    Ok(())
}

#[test]
fn writer_leaves_out_grace_notes_on_rests() -> GpResult<()> {
    let mut score = mscx_to_loaded_score(&parse_mscx(MSCX_NOTATION)?).score;
    let voice = present(
        present(
            score.score.tracks[0].measures.values_mut().next(),
            "measure",
        )?
        .voices
        .get_mut(&0),
        "voice",
    )?;
    // Move the chord's grace note onto the rest after it.
    let graces = std::mem::take(&mut voice.beats[0].grace_notes);
    voice.beats[1].grace_notes = graces;
    let xml = loaded_score_to_mscx(&score).raw_xml;
    assert!(!xml.contains("<acciaccatura/>"));
    Ok(())
}

#[test]
fn writer_leaves_out_grace_notes_without_a_pitch() -> GpResult<()> {
    let mut score = mscx_to_loaded_score(&parse_mscx(MSCX_NOTATION)?).score;
    let voice = present(
        present(
            score.score.tracks[0].measures.values_mut().next(),
            "measure",
        )?
        .voices
        .get_mut(&0),
        "voice",
    )?;
    present(voice.beats[0].grace_notes.first_mut(), "grace note")?
        .note
        .pitch = None;
    let xml = loaded_score_to_mscx(&score).raw_xml;
    // Its grace chord, of no other notes, is left out whole.
    assert!(!xml.contains("<acciaccatura/>"));
    Ok(())
}

#[test]
fn segment_elements_before_each_grace_chord_reach_the_main_chord() -> GpResult<()> {
    // A second grace chord, with a dynamic before it, after the first.
    let xml = MSCX_NOTATION.replacen(
        GRACE_CHORD,
        &format!(
            "{GRACE_CHORD}\n          <Dynamic><subtype>mf</subtype><velocity>80</velocity></Dynamic>\n          {GRACE_CHORD}"
        ),
        1,
    );
    assert_ne!(xml, MSCX_NOTATION);
    let outcome = mscx_to_loaded_score(&parse_mscx(&xml)?);
    let main = &beats(&outcome.score, 0, 0)?[0];
    assert_eq!(main.grace_notes.len(), 2);
    // The later dynamic holds; the earlier pp is reported.
    assert_eq!(main.dynamic, Some(Dynamic::MF));
    assert_eq!(
        outcome
            .report
            .get("Dynamic (before a grace chord, superseded)"),
        1
    );
    // What was written before the first grace chord is kept too.
    assert!(main.chord.is_some());
    assert!(!hairpins(main).is_empty());
    Ok(())
}

#[test]
fn guitar_pro_ties_are_linked_and_written() -> GpResult<()> {
    use crate::convert::optimized::legacy::legacy_song_to_loaded_score;
    use crate::model::legacy::song::Song;
    use crate::tests::common::read_file;

    let mut song = Song::default();
    song.read_gp5(&read_file("test/Demo v5.gp5".to_string()))?;
    let score = legacy_song_to_loaded_score(&song);
    let notes: Vec<_> = score
        .score
        .tracks
        .iter()
        .flat_map(|track| track.measures.values())
        .flat_map(|data| data.voices.values())
        .flat_map(|voice| &voice.beats)
        .flat_map(|beat| &beat.notes)
        .collect();
    let ends = notes
        .iter()
        .filter(|note| note.tie.is_some_and(TieType::ends_tie))
        .count();
    let starts = notes
        .iter()
        .filter(|note| note.tie.is_some_and(TieType::starts_tie))
        .count();
    // Each tied note is tied from the note before it on its string, whose
    // pitch it takes.
    assert!(ends > 0);
    assert_eq!(starts, ends);
    assert!(
        notes
            .iter()
            .filter(|note| note.tie.is_some_and(TieType::ends_tie))
            .all(|note| note.pitch.is_some())
    );
    let xml = loaded_score_to_mscx(&score).raw_xml;
    assert_eq!(xml.matches(r#"<Spanner type="Tie">"#).count(), 2 * ends);
    Ok(())
}

#[test]
fn jumps_and_markers_carry_over_to_guitar_pro_and_back() -> GpResult<()> {
    use crate::convert::legacy::loaded_score_to_legacy_song;
    use crate::convert::optimized::legacy::legacy_song_to_loaded_score;
    use crate::model::legacy::enums::DirectionSign;

    let score = mscx_to_loaded_score(&parse_mscx(MSCX_NOTATION)?).score;
    let song = loaded_score_to_legacy_song(&score);
    let directions: Vec<Vec<DirectionSign>> = song
        .measure_headers
        .iter()
        .map(|header| header.directions.clone())
        .collect();
    assert_eq!(
        directions,
        vec![
            vec![DirectionSign::Segno],
            vec![],
            vec![DirectionSign::DaSegnoAlFine]
        ]
    );
    let back = legacy_song_to_loaded_score(&song);
    let jumps: Vec<Vec<JumpKind>> = back
        .score
        .timeline
        .iter()
        .map(|measure| measure.navigation.iter().map(|nav| nav.kind).collect())
        .collect();
    assert_eq!(
        jumps,
        vec![
            vec![JumpKind::Segno],
            vec![],
            vec![JumpKind::DalSegnoAlFine]
        ]
    );

    // A jump taken out of the model stays out.
    let mut edited = back;
    if let Some(last) = edited.score.timeline.last_mut() {
        last.navigation.clear();
    }
    let song = loaded_score_to_legacy_song(&edited);
    let last = present(song.measure_headers.last(), "last bar")?;
    assert!(last.directions.is_empty());
    Ok(())
}

#[test]
fn the_first_voices_explicit_regular_barline_holds() -> GpResult<()> {
    let xml = MSCX_NOTATION.replacen(
        "<BarLine><subtype>double</subtype></BarLine>\n        </voice>",
        "<BarLine><subtype>normal</subtype></BarLine>\n        </voice>\n        <voice><Rest><durationType>measure</durationType></Rest><BarLine><subtype>double</subtype></BarLine></voice>",
        1,
    );
    assert_ne!(xml, MSCX_NOTATION);
    let score = mscx_to_loaded_score(&parse_mscx(&xml)?).score;
    let style = score.score.timeline[0]
        .barline_right
        .as_ref()
        .map_or(BarlineStyle::Regular, |barline| barline.style);
    assert_eq!(style, BarlineStyle::Regular);
    Ok(())
}

/// [`MSCX_NOTATION`] with a second voice in bar 1, with `dynamic` before
/// its chord.
fn with_second_voice(dynamic: &str) -> String {
    let xml = MSCX_NOTATION.replacen(
        "<BarLine><subtype>double</subtype></BarLine>\n        </voice>",
        &format!(
            "<BarLine><subtype>double</subtype></BarLine>\n        </voice>\n        <voice>{dynamic}<Chord><durationType>whole</durationType><Note><pitch>48</pitch><tpc>14</tpc></Note></Chord></voice>"
        ),
        1,
    );
    assert_ne!(xml, MSCX_NOTATION);
    xml
}

/// A GP track that stays at the default forte still marks it, once per
/// voice: MuseScore's own default is mf.
#[test]
fn guitar_pro_tracks_at_the_default_level_mark_it_once_per_voice() -> GpResult<()> {
    use crate::convert::legacy::dynamics::keep_dynamic_changes;

    let mut score = mscx_to_loaded_score(&parse_mscx(&with_second_voice(""))?).score;
    let measures = &mut score.score.tracks[0].measures;
    for beat in measures
        .values_mut()
        .flat_map(|data| data.voices.values_mut())
        .flat_map(|voice| voice.beats.iter_mut())
    {
        beat.dynamic = Some(Dynamic::F);
    }
    let voices: std::collections::HashSet<u8> = measures
        .values()
        .flat_map(|data| data.voices.iter())
        .filter(|(_, voice)| !voice.beats.is_empty())
        .map(|(&id, _)| id)
        .collect();
    keep_dynamic_changes(measures);
    let kept: Vec<&Dynamic> = measures
        .values()
        .flat_map(|data| data.voices.values())
        .flat_map(|voice| &voice.beats)
        .filter_map(|beat| beat.dynamic.as_ref())
        .collect();
    assert_eq!(kept.len(), voices.len());
    assert!(kept.iter().all(|dynamic| **dynamic == Dynamic::F));
    let xml = loaded_score_to_mscx(&score).raw_xml;
    assert!(xml.contains("<Dynamic>"));
    Ok(())
}

#[test]
fn guitar_pro_dynamics_in_several_voices_are_written_per_voice() -> GpResult<()> {
    use crate::convert::legacy::dynamics::keep_dynamic_changes;

    let mut score = mscx_to_loaded_score(&parse_mscx(&with_second_voice(""))?).score;
    let measures = &mut score.score.tracks[0].measures;
    // As read from GP: every beat's dynamic from its velocity. Voice 0
    // stays at the default forte while voice 1 plays piano.
    for data in measures.values_mut() {
        for (&voice, v) in data.voices.iter_mut() {
            for beat in &mut v.beats {
                beat.dynamic = Some(if voice == 0 { Dynamic::F } else { Dynamic::P });
            }
        }
    }
    keep_dynamic_changes(measures);
    let bar1 = present(measures.values().next(), "bar 1")?;
    let first = |voice: u8| -> GpResult<Option<Dynamic>> {
        let beats = &present(bar1.voices.get(&voice), "voice")?.beats;
        Ok(present(beats.first(), "beat")?.dynamic.clone())
    };
    // Each voice starts at its own level, and changes only where it does.
    assert_eq!(first(0)?, Some(Dynamic::F));
    assert_eq!(first(1)?, Some(Dynamic::P));
    let kept = measures
        .values()
        .flat_map(|data| data.voices.values())
        .flat_map(|voice| &voice.beats)
        .filter(|beat| beat.dynamic.is_some())
        .count();
    assert_eq!(kept, 2);
    // Each holds for its own voice only.
    let xml = loaded_score_to_mscx(&score).raw_xml;
    assert_eq!(
        xml.matches("<voiceAssignment>currentVoiceOnly</voiceAssignment>")
            .count(),
        2
    );
    Ok(())
}

#[test]
fn dynamics_per_voice_round_trip() -> GpResult<()> {
    const VOICE_ONLY: &str = "<voiceAssignment>currentVoiceOnly</voiceAssignment>";
    // Bar 1's pp and sempre p are for voice 1 only, its second voice's mf
    // for that voice only.
    let xml = with_second_voice(&format!(
        "<Dynamic><subtype>mf</subtype><velocity>80</velocity>{VOICE_ONLY}</Dynamic>"
    ))
    .replacen(
        "<velocity>33</velocity></Dynamic>",
        &format!("<velocity>33</velocity>{VOICE_ONLY}</Dynamic>"),
        1,
    )
    .replacen(
        "<text>sempre <i>p</i></text></Dynamic>",
        &format!("<text>sempre <i>p</i></text>{VOICE_ONLY}</Dynamic>"),
        1,
    );
    let outcome = mscx_to_loaded_score(&parse_mscx(&xml)?);
    assert_eq!(outcome.report.get("Dynamic (voice assignment)"), 0);
    let written = loaded_score_to_mscx(&outcome.score).raw_xml;
    assert_eq!(written.matches(VOICE_ONLY).count(), 3);

    // A dynamic for all voices among them can't be held.
    let mixed = xml.replacen(VOICE_ONLY, "", 1);
    let outcome = mscx_to_loaded_score(&parse_mscx(&mixed)?);
    assert_eq!(outcome.report.get("Dynamic (voice assignment)"), 1);
    Ok(())
}

#[test]
fn a_guitar_pro_tied_note_after_a_rest_keeps_its_pitch() -> GpResult<()> {
    use crate::convert::optimized::legacy::legacy_song_to_loaded_score;
    use crate::model::legacy::{enums::BeatStatus, enums::NoteType, song::Song};
    use crate::tests::common::read_file;

    let mut song = Song::default();
    song.read_gp5(&read_file("test/Demo v5.gp5".to_string()))?;
    // The first tied note with a beat before it in its voice, which
    // becomes a rest: nothing is left to tie it from.
    let mut found = None;
    'search: for (t, track) in song.tracks.iter().enumerate() {
        for (m, measure) in track.measures.iter().enumerate() {
            for (v, voice) in measure.voices.iter().enumerate() {
                for (b, beat) in voice.beats.iter().enumerate().skip(1) {
                    if beat.notes.iter().any(|note| note.kind == NoteType::Tie) {
                        found = Some((t, m, v, b));
                        break 'search;
                    }
                }
            }
        }
    }
    let (t, m, v, b) = present(found, "a tied note after another beat")?;
    let before = &mut song.tracks[t].measures[m].voices[v].beats[b - 1];
    before.notes.clear();
    before.status = BeatStatus::Rest;

    let score = legacy_song_to_loaded_score(&song);
    let data = present(score.score.tracks[t].measures.values().nth(m), "measure")?;
    let beat = present(
        present(data.voices.get(&(v as u8)), "voice")?.beats.get(b),
        "beat",
    )?;
    let tied: Vec<_> = beat
        .notes
        .iter()
        .filter(|note| note.tie.is_some())
        .collect();
    assert!(!tied.is_empty());
    assert!(tied.iter().all(|note| note.pitch.is_some()));
    Ok(())
}

#[test]
fn a_hairpin_can_end_in_another_voice() -> GpResult<()> {
    // Bar 1's crescendo ends on bar 2's second voice instead of its first.
    let end = r#"<Spanner type="HairPin"><prev><location><measures>-1</measures></location></prev></Spanner>"#;
    let xml = MSCX_NOTATION.replacen(&format!("{end}\n"), "", 1).replacen(
        "<Tremolo><subtype>r16</subtype></Tremolo><Note><pitch>64</pitch><tpc>18</tpc></Note></Chord>\n        </voice>",
        &format!(
            "<Tremolo><subtype>r16</subtype></Tremolo><Note><pitch>64</pitch><tpc>18</tpc></Note></Chord>\n        </voice>\n        <voice><Chord><durationType>whole</durationType><Note><pitch>48</pitch><tpc>14</tpc></Note></Chord>\n          {end}</voice>"
        ),
        1,
    );
    assert_eq!(xml.matches(end).count(), 1);
    assert!(xml.contains("<pitch>48</pitch><tpc>14</tpc></Note></Chord>\n          <Spanner"));
    let outcome = mscx_to_loaded_score(&parse_mscx(&xml)?);
    assert_eq!(outcome.report.get("HairPin (unpaired end)"), 0);
    assert_eq!(outcome.report.get("HairPin (unterminated)"), 0);
    let crescendo = hairpins(&beats(&outcome.score, 0, 0)?[0])[0].0;
    let bar2 = present(
        outcome.score.score.tracks[0].measures.values().nth(1),
        "bar 2",
    )?;
    let second = &present(bar2.voices.get(&1), "voice 2")?.beats;
    assert_eq!(hairpins(&second[0]), vec![(crescendo, HairpinKind::Stop)]);
    Ok(())
}

#[test]
fn a_measures_signs_all_reach_guitar_pro_files_and_back() -> GpResult<()> {
    use crate::convert::legacy::loaded_score_to_legacy_song;
    use crate::model::legacy::{enums::DirectionSign, song::Song};

    // Bar 1 has a segno and, after it, a D.C. al Fine.
    let segno = "<label>segno</label></Marker>";
    let xml = MSCX_NOTATION.replacen(
        segno,
        &format!("{segno}\n        <Jump><text>D.C. al Fine</text><jumpTo>start</jumpTo><playUntil>fine</playUntil><continueAt></continueAt></Jump>"),
        1,
    );
    assert_ne!(xml, MSCX_NOTATION);
    let score = mscx_to_loaded_score(&parse_mscx(&xml)?).score;
    let song = loaded_score_to_legacy_song(&score);
    let both = vec![DirectionSign::Segno, DirectionSign::DaCapoAlFine];
    let first = |song: &Song| song.measure_headers.first().map(|h| h.directions.clone());
    assert_eq!(first(&song), Some(both.clone()));

    // Through a GP7 file (a `<Target>` and a `<Jump>`), and a GP5 one
    // (written from a GP5 file's song, which GP5 can hold).
    let mut gp7 = Song::default();
    gp7.read_gp(&song.write_gp()?)?;
    assert_eq!(first(&gp7), Some(both.clone()));
    let mut gp5 = Song::default();
    gp5.read_gp5(&crate::tests::common::read_file(
        "test/Demo v5.gp5".to_string(),
    ))?;
    // GP5 holds each sign in one measure: only bar 1's here.
    for header in &mut gp5.measure_headers {
        header.directions.clear();
    }
    if let Some(header) = gp5.measure_headers.first_mut() {
        header.directions = both.clone();
    }
    let mut back = Song::default();
    back.read_gp5(&gp5.write(gp5.version.number, None)?)?;
    assert_eq!(first(&back), Some(both));
    Ok(())
}

/// Scores saved before the navigation had the direction signs kept all
/// of them in `gp.direction.N`: loading one moves those with a jump into
/// the navigation, and a GP export still writes them all.
#[test]
fn a_saved_scores_direction_signs_reach_guitar_pro_files() -> GpResult<()> {
    use crate::convert::{
        legacy::loaded_score_to_legacy_song, optimized::legacy::legacy_song_to_loaded_score,
    };
    use crate::model::optimized::timeline::JumpKind;

    let song = crate::tests::common::read_gp7("test/directions.gp");
    let expected: Vec<_> = song
        .measure_headers
        .iter()
        .map(|header| header.directions.clone())
        .collect();
    assert!(expected.iter().any(|signs| !signs.is_empty()));

    // The score as saved then: no jumps in its navigation, every sign in
    // the miscellaneous fields.
    let mut score = legacy_song_to_loaded_score(&song);
    for measure in &mut score.score.timeline {
        measure
            .navigation
            .retain(|ev| matches!(ev.kind, JumpKind::RepeatOpen | JumpKind::RepeatClose));
    }
    let misc = &mut present(
        score.score.metadata.identification.as_mut(),
        "identification",
    )?
    .miscellaneous;
    misc.retain(|(key, _)| !key.starts_with("gp.direction."));
    for (index, signs) in expected.iter().enumerate() {
        if !signs.is_empty() {
            let names: Vec<_> = signs.iter().map(|sign| sign.name()).collect();
            misc.push((format!("gp.direction.{index}"), names.join(",")));
        }
    }
    let json = crate::tests::common::unversioned_save(&score.score).map_err(|e| e.to_string())?;
    score.score = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let jumps = |score: &crate::model::optimized::LoadedScore| -> Vec<Vec<JumpKind>> {
        score
            .score
            .timeline
            .iter()
            .map(|m| m.navigation.iter().map(|ev| ev.kind).collect())
            .collect()
    };
    assert_eq!(jumps(&score), jumps(&legacy_song_to_loaded_score(&song)));
    let exported: Vec<_> = loaded_score_to_legacy_song(&score)
        .measure_headers
        .iter()
        .map(|header| header.directions.clone())
        .collect();
    assert_eq!(exported, expected);
    Ok(())
}

/// A jump removed from a score's navigation stays removed on a GP export,
/// even with its sign left in `gp.direction.N` from an older save.
#[test]
fn a_removed_jump_stays_removed_on_a_guitar_pro_export() -> GpResult<()> {
    use crate::convert::{
        legacy::loaded_score_to_legacy_song, optimized::legacy::legacy_song_to_loaded_score,
    };
    use crate::model::optimized::timeline::JumpKind;

    let song = crate::tests::common::read_gp7("test/directions.gp");
    let mut score = legacy_song_to_loaded_score(&song);
    let is_jump = |kind: JumpKind| !matches!(kind, JumpKind::RepeatOpen | JumpKind::RepeatClose);
    let jumps: Vec<usize> = score
        .score
        .timeline
        .iter()
        .enumerate()
        .filter(|(_, m)| m.navigation.iter().any(|ev| is_jump(ev.kind)))
        .map(|(index, _)| index)
        .collect();
    assert!(jumps.len() >= 2, "{jumps:?}");
    // Remove the first measure's signs, leaving them in the misc fields.
    let removed = jumps[0];
    score.score.timeline[removed]
        .navigation
        .retain(|ev| !is_jump(ev.kind));
    let names: Vec<_> = song.measure_headers[removed]
        .directions
        .iter()
        .map(|sign| sign.name())
        .collect();
    present(
        score.score.metadata.identification.as_mut(),
        "identification",
    )?
    .miscellaneous
    .push((format!("gp.direction.{removed}"), names.join(",")));
    let exported = loaded_score_to_legacy_song(&score);
    let left: Vec<_> = exported.measure_headers[removed]
        .directions
        .iter()
        .filter(|sign| crate::model::directions::direction_to_jump(sign).is_some())
        .collect();
    assert!(left.is_empty(), "{left:?}");
    Ok(())
}

#[test]
fn segment_elements_before_trailing_grace_chords_are_reported() -> GpResult<()> {
    // A dynamic, then a grace chord, after the measure's last beat.
    let xml = MSCX_NOTATION.replacen(
        r#"<Rest><durationType>half</durationType></Rest>"#,
        &format!(
            "<Rest><durationType>half</durationType></Rest>\n          <Dynamic><subtype>f</subtype></Dynamic>\n          {GRACE_CHORD}"
        ),
        1,
    );
    assert_ne!(xml, MSCX_NOTATION);
    let outcome = mscx_to_loaded_score(&parse_mscx(&xml)?);
    assert_eq!(outcome.report.get("Grace (without a main chord)"), 1);
    assert_eq!(outcome.report.get("Dynamic (after the last beat)"), 1);
    Ok(())
}

/// A `<Jump>` to a segno is a D.S.; one to another label (a coda, a
/// user's marker) is one only if its text says so.
#[test]
fn jumps_to_labels_other_than_a_segno_are_no_dal_segno() {
    use crate::convert::mscz::notation::jump_kind;
    use crate::model::mscz::mscx::{MscxJump, MscxMeasureMark};
    use crate::model::optimized::timeline::JumpKind;
    let kind = |text: &str, jump_to: &str| {
        jump_kind(&MscxMeasureMark::Jump(MscxJump {
            text: text.to_string(),
            jump_to: jump_to.to_string(),
            play_until: String::new(),
            continue_at: String::new(),
        }))
    };
    assert_eq!(kind("", "segno"), Some(JumpKind::DalSegno));
    // The second segno is no segno (Dal Doppio Segno), nor is D.S.S.'s
    // text a D.S.
    assert_eq!(kind("", "varsegno"), None);
    assert_eq!(kind("D.S.S.", "mysegno"), None);
    assert_eq!(kind("", "codab"), None);
    assert_eq!(kind("To the bridge", "bridge"), None);
    assert_eq!(kind("D.S.", "mysegno"), Some(JumpKind::DalSegno));
}

/// MuseScore's second segno and doppia coda (as 4.5 writes them) aren't
/// merged into the segno and coda: they're reported.
#[test]
fn doubled_segno_and_coda_are_not_merged() -> GpResult<()> {
    use crate::convert::mscz::notation::jump_kind;
    use crate::model::mscz::mscx::{MscxJump, MscxMarker, MscxMeasureMark};
    let marker = |label: &str| {
        MscxMeasureMark::Marker(MscxMarker {
            label: label.to_string(),
            text: String::new(),
        })
    };
    let jump = |text: &str, jump_to: &str, play_until: &str| {
        MscxMeasureMark::Jump(MscxJump {
            text: text.to_string(),
            jump_to: jump_to.to_string(),
            play_until: play_until.to_string(),
            continue_at: "codab".to_string(),
        })
    };
    for mark in [
        marker("varsegno"),
        marker("varcoda"),
        marker("codetta"),
        jump("Dal Doppio Segno", "varsegno", "end"),
        jump("D.S. al Doppia Coda", "segno", "varcoda"),
        jump("D.C. al Doppia Coda", "start", "varcoda"),
    ] {
        assert_eq!(jump_kind(&mark), None, "{mark:?}");
    }
    assert_eq!(
        jump_kind(&jump("D.S. al Coda", "segno", "coda")),
        Some(JumpKind::DalSegnoAlCoda)
    );

    let doubled = r#"<Marker><text><sym>segno</sym></text><label>varsegno</label></Marker>"#;
    let jump_xml = r#"<Jump><text>D.S. al Fine</text><jumpTo>segno</jumpTo><playUntil>fine</playUntil><continueAt></continueAt></Jump>"#;
    let mscx = MSCX_NOTATION.replacen(jump_xml, &format!("{jump_xml}{doubled}"), 1);
    assert_ne!(mscx, MSCX_NOTATION);
    let outcome = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    assert_eq!(
        outcome
            .report
            .get("Jump/Marker (doppio segno or doppia coda)"),
        1
    );
    // Kept as GP's sign, so the repeats analysis flags the jump playback
    // can't follow, as for a GP file.
    let kept = outcome
        .score
        .score
        .metadata
        .identification
        .as_ref()
        .map(|id| id.miscellaneous.clone())
        .unwrap_or_default();
    assert!(
        kept.iter()
            .any(|(key, value)| key.starts_with("gp.direction.") && value == "SegnoSegno"),
        "{kept:?}"
    );
    let dss = r#"<Jump><text>Dal Doppio Segno</text><jumpTo>varsegno</jumpTo><playUntil>end</playUntil><continueAt></continueAt></Jump>"#;
    let mscx = MSCX_NOTATION.replacen(jump_xml, &format!("{jump_xml}{dss}"), 1);
    let outcome = mscx_to_loaded_score(&parse_mscx(&mscx)?);
    let song = crate::convert::legacy::loaded_score_to_legacy_song(&outcome.score);
    assert!(crate::analysis::repeats::has_unfollowed_jumps(
        &song.measure_headers
    ));

    use crate::convert::mscz::notation::doubled_direction;
    use crate::model::legacy::enums::DirectionSign;
    assert_eq!(
        doubled_direction(&jump("Dal Doppio Segno", "varsegno", "end")),
        Some(DirectionSign::DaSegnoSegno)
    );
    assert_eq!(
        doubled_direction(&jump("D.S. al Doppia Coda", "segno", "varcoda")),
        Some(DirectionSign::DaSegnoAlDoubleCoda)
    );
    assert_eq!(
        doubled_direction(&jump("D.C. al Doppia Coda", "start", "varcoda")),
        Some(DirectionSign::DaCapoAlDoubleCoda)
    );
    assert_eq!(
        doubled_direction(&marker("codetta")),
        Some(DirectionSign::DoubleCoda)
    );
    assert_eq!(
        doubled_direction(&jump("D.S. al Coda", "segno", "coda")),
        None
    );
    Ok(())
}

/// A measure's signs (segno, coda) come before its repeat close and its
/// jumps after, as from GP: playback takes a measure's first jump.
#[test]
fn jumps_are_ordered_around_a_measures_repeat_as_from_guitar_pro() -> GpResult<()> {
    let jump = r#"<Jump><text>D.S. al Fine</text><jumpTo>segno</jumpTo><playUntil>fine</playUntil><continueAt></continueAt></Jump>"#;
    assert!(MSCX_NOTATION.contains(jump));
    let mscx = MSCX_NOTATION.replacen(
        jump,
        &format!(
            r#"{jump}<Marker><text><sym>coda</sym></text><label>codab</label></Marker><endRepeat>2</endRepeat>"#
        ),
        1,
    );
    let score = mscx_to_loaded_score(&parse_mscx(&mscx)?).score;
    let kinds: Vec<JumpKind> = score.score.timeline[2]
        .navigation
        .iter()
        .map(|event| event.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            JumpKind::Coda,
            JumpKind::RepeatClose,
            JumpKind::DalSegnoAlFine
        ]
    );
    Ok(())
}

/// Two dynamics or chord symbols before one beat: the beat holds the last,
/// and the earlier one is reported.
#[test]
fn several_dynamics_or_chord_symbols_before_one_beat_report_the_earlier() -> GpResult<()> {
    const PP: &str = "<Dynamic><subtype>pp</subtype><velocity>33</velocity></Dynamic>";
    const HARMONY: &str = "<Harmony><root>14</root><name>m7</name><base>11</base></Harmony>";
    let xml = MSCX_NOTATION
        .replacen(
            PP,
            &format!("<Dynamic><subtype>sfz</subtype></Dynamic>\n          {PP}"),
            1,
        )
        .replacen(
            HARMONY,
            &format!("<Harmony><root>15</root><name>7</name></Harmony>\n          {HARMONY}"),
            1,
        );
    assert_ne!(xml, MSCX_NOTATION);
    let outcome = mscx_to_loaded_score(&parse_mscx(&xml)?);
    let main = &beats(&outcome.score, 0, 0)?[0];
    assert_eq!(main.dynamic, Some(Dynamic::PP));
    assert_eq!(
        present(main.chord.as_ref(), "chord symbol")?.kind,
        "m7".to_string()
    );
    let report = &outcome.report;
    assert_eq!(
        report.get("Dynamic (several before one beat, last kept)"),
        1
    );
    assert_eq!(
        report.get("Harmony (several before one beat, last kept)"),
        1
    );
    Ok(())
}
