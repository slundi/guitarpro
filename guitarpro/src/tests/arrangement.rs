//! Playback order: repeats, voltas and jumps (`Arrangement::resolve`).

use crate::model::optimized::global::MeasureIndex;
use crate::model::optimized::timeline::{
    Arrangement, Barline, BarlineStyle, Ending, EndingKind, JumpKind, MeasureDef, NavigationEvent,
    ending_numbers, ending_spans, set_ending,
};
use crate::tests::common::TestResult;

fn measure(idx: u16, navigation: Vec<NavigationEvent>) -> MeasureDef {
    MeasureDef {
        index: MeasureIndex(idx),
        tempo: None,
        time_signature: None,
        key_signature: None,
        marker: None,
        navigation,
        tick_resolution: 960,
        duration_ticks: 3840,
        barline_left: None,
        barline_right: None,
        gp_beams: None,
        gp_fermatas: Vec::new(),
        gp_free_time: false,
    }
}

fn event(kind: JumpKind, repeat_count: Option<u8>) -> NavigationEvent {
    NavigationEvent {
        measure_index: MeasureIndex(0),
        kind,
        repeat_count,
    }
}

fn nav(kind: JumpKind) -> NavigationEvent {
    event(kind, None)
}

fn repeat_close(count: u8) -> NavigationEvent {
    event(JumpKind::RepeatClose, Some(count))
}

/// `measure` as a one-measure volta bracket over `numbers`.
fn in_ending(mut measure: MeasureDef, numbers: &[u8]) -> MeasureDef {
    let barline = |kind| {
        Some(Barline {
            style: BarlineStyle::Regular,
            ending: Some(Ending {
                numbers: numbers.to_vec(),
                text: None,
                kind,
            }),
        })
    };
    measure.barline_left = barline(EndingKind::Start);
    measure.barline_right = barline(EndingKind::Stop);
    measure
}

fn idx(n: u16) -> MeasureIndex {
    MeasureIndex(n)
}

fn order(timeline: &[MeasureDef]) -> Vec<u16> {
    Arrangement::resolve(timeline)
        .order
        .into_iter()
        .map(|index| index.0)
        .collect()
}

/// Long nested repeats scale the resolver's guard, but not past its cap:
/// a crafted score can't make the played order huge.
#[test]
fn the_resolver_guard_is_capped_for_long_nested_repeats() {
    let n: u16 = 40_000;
    let timeline: Vec<MeasureDef> = (0..n)
        .map(|index| {
            let mut navigation = Vec::new();
            if index < 3 {
                navigation.push(nav(JumpKind::RepeatOpen));
            }
            if index >= n - 3 {
                navigation.push(repeat_close(255));
            }
            measure(index, navigation)
        })
        .collect();
    let arrangement = Arrangement::resolve(&timeline);
    assert!(arrangement.truncated);
    assert!(arrangement.order.len() <= usize::from(n) * 32);
}

/// Keeping only the first bars keeps the same ones and counts them all.
#[test]
fn resolving_the_first_bars_counts_them_all() {
    let timeline = vec![
        measure(0, vec![nav(JumpKind::RepeatOpen)]),
        measure(1, vec![repeat_close(200)]),
        measure(2, vec![]),
    ];
    let full = Arrangement::resolve(&timeline);
    assert_eq!(full.played, full.order.len());
    assert_eq!(full.played, 401);
    let first = Arrangement::resolve_first(&timeline, 10);
    assert_eq!(first.order, full.order[..10]);
    assert_eq!(first.passes, full.passes[..10]);
    assert_eq!(first.played, full.played);
}

/// A D.C. al Coda return pass skips the repeat, straight to its last
/// ending, and takes the To Coda there; the 1st ending's isn't played.
#[test]
fn a_return_pass_takes_the_to_coda_in_the_last_ending() {
    let timeline = vec![
        measure(0, vec![nav(JumpKind::RepeatOpen)]),
        in_ending(measure(1, vec![repeat_close(2)]), &[1]),
        in_ending(measure(2, vec![nav(JumpKind::ToCoda)]), &[2]),
        measure(3, vec![nav(JumpKind::DaCapoAlCoda)]),
        measure(4, vec![nav(JumpKind::Coda)]),
    ];
    assert_eq!(order(&timeline), vec![0, 1, 0, 2, 3, 0, 2, 4]);
}

/// A D.C. al Fine return pass plays the last ending, where the Fine is.
#[test]
fn a_return_pass_ends_at_the_fine_in_the_last_ending() {
    let timeline = vec![
        measure(0, vec![nav(JumpKind::RepeatOpen)]),
        in_ending(measure(1, vec![repeat_close(2)]), &[1]),
        in_ending(measure(2, vec![nav(JumpKind::Fine)]), &[2]),
        measure(3, vec![nav(JumpKind::DaCapoAlFine)]),
    ];
    assert_eq!(order(&timeline), vec![0, 1, 0, 2, 3, 0, 2]);
}

/// `|: m0 [1] m1 :| [2] m2 m3 :|`: m3's repeat has no start barline, so it
/// starts after the first repeat's endings. Pass 2 skips the 1st ending,
/// close and all, which ends the first repeat there.
#[test]
fn a_repeat_without_a_start_follows_the_last_ones_endings() {
    let timeline = vec![
        measure(0, vec![nav(JumpKind::RepeatOpen)]),
        in_ending(measure(1, vec![repeat_close(2)]), &[1]),
        in_ending(measure(2, vec![]), &[2]),
        measure(3, vec![repeat_close(2)]),
    ];
    assert_eq!(order(&timeline), vec![0, 1, 0, 2, 3, 3]);
}

/// A repeat with no start barline at all repeats from the beginning.
#[test]
fn a_lone_repeat_close_repeats_from_the_start() {
    let timeline = vec![measure(0, vec![]), measure(1, vec![repeat_close(2)])];
    assert_eq!(order(&timeline), vec![0, 1, 0, 1]);
}

/// A nested repeat's last ending is played on its final pass: the
/// endings after it ends match its pass, not the enclosing repeat's.
#[test]
fn a_nested_repeat_plays_its_last_ending() {
    let timeline = vec![
        measure(0, vec![nav(JumpKind::RepeatOpen)]),
        measure(1, vec![nav(JumpKind::RepeatOpen)]),
        in_ending(measure(2, vec![repeat_close(2)]), &[1]),
        in_ending(measure(3, vec![]), &[2]),
        measure(4, vec![repeat_close(2)]),
    ];
    assert_eq!(order(&timeline), vec![0, 1, 2, 1, 3, 4, 0, 1, 2, 1, 3, 4]);
}

/// The closes in a finished nested repeat's skipped endings don't end
/// the enclosing repeat.
#[test]
fn a_nested_repeats_skipped_endings_keep_the_outer_repeat() {
    let timeline = vec![
        measure(0, vec![nav(JumpKind::RepeatOpen)]),
        measure(1, vec![nav(JumpKind::RepeatOpen)]),
        in_ending(measure(2, vec![repeat_close(3)]), &[1]),
        in_ending(measure(3, vec![repeat_close(3)]), &[2]),
        in_ending(measure(4, vec![]), &[3]),
        measure(5, vec![repeat_close(2)]),
    ];
    let once = [0, 1, 2, 1, 3, 1, 4, 5];
    assert_eq!(order(&timeline), [once, once].concat());
}

/// A nested repeat closing right before the enclosing repeat's endings
/// leaves them to the enclosing repeat's passes.
#[test]
fn an_inner_repeat_before_the_outer_endings_leaves_them_to_the_outer() {
    let timeline = vec![
        measure(0, vec![nav(JumpKind::RepeatOpen)]),
        measure(1, vec![nav(JumpKind::RepeatOpen)]),
        measure(2, vec![repeat_close(2)]),
        in_ending(measure(3, vec![repeat_close(2)]), &[1]),
        in_ending(measure(4, vec![]), &[2]),
        measure(5, vec![]),
    ];
    assert_eq!(
        order(&timeline),
        vec![0, 1, 2, 1, 2, 3, 0, 1, 2, 1, 2, 4, 5]
    );
}

/// A Fine in the 1st ending isn't taken on a D.C. al Fine return pass:
/// the pass plays only the last ending.
#[test]
fn a_return_pass_skips_a_fine_in_the_first_ending() {
    let timeline = vec![
        measure(0, vec![nav(JumpKind::RepeatOpen)]),
        in_ending(measure(1, vec![nav(JumpKind::Fine), repeat_close(2)]), &[1]),
        in_ending(measure(2, vec![]), &[2]),
        measure(3, vec![nav(JumpKind::DaCapoAlFine)]),
    ];
    assert_eq!(order(&timeline), vec![0, 1, 0, 2, 3, 0, 2, 3]);
}

/// A repeat with only a 1st ending: the D.C. return pass skips it, as
/// the repeat's final pass does, rather than playing it as the last.
#[test]
fn a_return_pass_skips_a_lone_first_ending() {
    let timeline = vec![
        measure(0, vec![nav(JumpKind::RepeatOpen)]),
        in_ending(measure(1, vec![repeat_close(2)]), &[1]),
        measure(2, vec![]),
        measure(3, vec![nav(JumpKind::DaCapo)]),
    ];
    assert_eq!(order(&timeline), vec![0, 1, 0, 2, 3, 0, 2, 3]);
}

/// 45i: `m0 [1. m1 |: m2 :| m3 :| [2. |: m4 :| m5] m6`: the repeat inside
/// the 1st ending doesn't end it, and the close after it repeats from the
/// start; the 2nd ending's own repeat leaves it to the final pass.
#[test]
fn a_repeat_inside_an_ending_keeps_the_enclosing_one() {
    let mut timeline = vec![
        measure(0, vec![]),
        measure(1, vec![]),
        measure(2, vec![nav(JumpKind::RepeatOpen), repeat_close(2)]),
        measure(3, vec![repeat_close(2)]),
        measure(4, vec![nav(JumpKind::RepeatOpen), repeat_close(2)]),
        measure(5, vec![]),
        measure(6, vec![]),
    ];
    set_ending(&mut timeline, 1, 3, &[1], None, true);
    set_ending(&mut timeline, 4, 5, &[2], None, false);
    assert_eq!(order(&timeline), vec![0, 1, 2, 2, 3, 0, 4, 4, 5, 6]);
}

/// 45d: five multi-measure endings of a repeat with no start barline,
/// each but the last closing it: every ending is played, after the start.
#[test]
fn a_repeat_plays_all_of_its_multi_measure_endings() {
    let mut timeline: Vec<_> = (0..12).map(|i| measure(i, vec![])).collect();
    for pos in [1, 4, 8, 9] {
        timeline[pos].navigation.push(repeat_close(2));
    }
    set_ending(&mut timeline, 1, 1, &[1], None, true);
    set_ending(&mut timeline, 2, 4, &[2], None, true);
    set_ending(&mut timeline, 5, 8, &[3], None, true);
    set_ending(&mut timeline, 9, 9, &[4], None, true);
    set_ending(&mut timeline, 10, 10, &[5], None, false);
    assert_eq!(
        order(&timeline),
        vec![0, 1, 0, 2, 3, 4, 0, 5, 6, 7, 8, 0, 9, 0, 10, 11]
    );
}

/// Scores saved before endings were barline endings give them as a GP
/// pass bitmask (`volta`) on a `RepeatClose`, one without a count standing
/// for just the ending: read as endings, without that close.
#[test]
fn a_saved_scores_voltas_are_read_as_endings() -> TestResult {
    let event = |kind: &str, count: Option<u8>, volta: Option<u8>| {
        serde_json::json!({
            "measure_index": 0, "kind": kind, "repeat_count": count,
            "volta": volta, "volta_last": false,
        })
    };
    let measure = |index: u16, navigation: Vec<serde_json::Value>| {
        serde_json::json!({
            "index": index, "tempo": null, "time_signature": null,
            "key_signature": null, "marker": null, "navigation": navigation,
            "tick_resolution": 960, "duration_ticks": 3840,
            "barline_left": null, "barline_right": null,
        })
    };
    let timeline = serde_json::json!([
        measure(0, vec![event("RepeatOpen", None, None)]),
        measure(1, vec![event("RepeatClose", Some(2), Some(0b01))]),
        measure(2, vec![event("RepeatClose", None, Some(0b10))]),
        measure(3, vec![event("RepeatClose", None, Some(0b10))]),
        measure(4, vec![]),
    ]);
    #[derive(serde::Deserialize)]
    struct Saved {
        #[serde(
            deserialize_with = "crate::model::optimized::timeline::deserialize_timeline_alone"
        )]
        timeline: Vec<MeasureDef>,
    }
    let saved: Saved = serde_json::from_value(serde_json::json!({ "timeline": timeline }))?;
    let timeline = saved.timeline;
    assert!(timeline[2].navigation.is_empty());
    assert_eq!(
        ending_numbers(&timeline),
        vec![None, Some(&[1u8][..]), Some(&[2][..]), Some(&[2][..]), None]
    );
    assert_eq!(order(&timeline), vec![0, 1, 0, 2, 3, 4]);
    Ok(())
}

/// The MusicXML test suite's repeat fixtures play in their documented
/// order (1-based measure numbers), also after the MSCZ and GP round trips.
#[test]
fn the_repeat_fixtures_play_in_order_across_formats() -> TestResult {
    use crate::convert::legacy::loaded_score_to_legacy_song;
    use crate::convert::mscz::{loaded_score_to_mscx, mscx_to_loaded_score};
    use crate::convert::optimized::{
        legacy::legacy_song_to_loaded_score, score_partwise_to_loaded_score,
    };
    use crate::model::optimized::LoadedScore;
    use crate::tests::common::read_musicxml;

    let fixtures: [(&str, &[u16]); 7] = [
        ("45a-SimpleRepeat", &[1, 1, 1, 1, 1, 2]),
        ("45b-RepeatWithAlternatives", &[1, 2, 1, 3, 4]),
        (
            "45c-SimpleRepeat-Nested",
            &[1, 2, 3, 2, 3, 2, 3, 2, 3, 2, 3, 4, 5, 6, 7, 8],
        ),
        (
            "45d-Repeats-MultipleEndings",
            &[1, 2, 1, 3, 4, 5, 1, 6, 7, 8, 9, 1, 10, 1, 11, 12],
        ),
        (
            "45e-Repeats-Combination",
            &[1, 2, 1, 3, 4, 5, 5, 6, 7, 6, 8, 9, 8, 9, 10],
        ),
        ("45h-Repeats-Partial", &[1, 2, 3, 1, 2, 3, 4, 5]),
        ("45i-Repeats-Nested", &[1, 2, 3, 3, 4, 1, 5, 5, 6, 7]),
    ];
    let played = |score: &LoadedScore| -> Vec<u16> {
        order(&score.score.timeline).iter().map(|i| i + 1).collect()
    };
    for (name, expected) in fixtures {
        let loaded = score_partwise_to_loaded_score(&read_musicxml(&format!("{name}.xml"))?);
        let mscz = mscx_to_loaded_score(&loaded_score_to_mscx(&loaded)).score;
        let gp = legacy_song_to_loaded_score(&loaded_score_to_legacy_song(&loaded));
        assert_eq!(played(&loaded), expected, "{name}");
        assert_eq!(played(&mscz), expected, "{name} after MSCZ");
        assert_eq!(played(&gp), expected, "{name} after GP");
    }
    Ok(())
}

/// Old scores from MusicXML give an ending by its first number, on its
/// last measure, marked `volta_last`: read as numbers, not GP bitmasks,
/// and the barline endings they kept stay.
#[test]
fn a_saved_musicxml_scores_voltas_are_ending_numbers() -> TestResult {
    let event = |kind: &str, count: Option<u8>, volta: Option<u8>| {
        serde_json::json!({
            "measure_index": 0, "kind": kind, "repeat_count": count,
            "volta": volta, "volta_last": volta.is_some(),
        })
    };
    let ending = |numbers: &[u8], kind: &str| {
        serde_json::json!({
            "style": "Regular",
            "ending": { "numbers": numbers, "text": null, "kind": kind },
        })
    };
    let measure = |index: u16, navigation: Vec<serde_json::Value>| {
        serde_json::json!({
            "index": index, "tempo": null, "time_signature": null,
            "key_signature": null, "marker": null, "navigation": navigation,
            "tick_resolution": 960, "duration_ticks": 3840,
            "barline_left": null, "barline_right": null,
        })
    };
    let mut kept = measure(4, vec![event("RepeatClose", Some(2), Some(1))]);
    kept["barline_left"] = ending(&[1, 2], "Start");
    kept["barline_right"] = ending(&[1, 2], "Stop");
    let timeline = serde_json::json!([
        measure(0, vec![event("RepeatOpen", None, None)]),
        measure(1, vec![event("RepeatClose", Some(3), Some(1))]),
        measure(2, vec![event("RepeatClose", Some(3), Some(2))]),
        measure(3, vec![event("RepeatClose", None, Some(3))]),
        kept,
    ]);
    #[derive(serde::Deserialize)]
    struct Saved {
        #[serde(
            deserialize_with = "crate::model::optimized::timeline::deserialize_timeline_alone"
        )]
        timeline: Vec<MeasureDef>,
    }
    let saved: Saved = serde_json::from_value(serde_json::json!({ "timeline": timeline }))?;
    assert_eq!(
        ending_numbers(&saved.timeline),
        vec![
            None,
            Some(&[1u8][..]),
            Some(&[2][..]),
            Some(&[3][..]),
            Some(&[1, 2][..])
        ]
    );
    Ok(())
}

/// An ending without a stop closes at its backward repeat, also when the
/// next ending starts on the very next measure.
#[test]
fn an_ending_closes_at_its_repeat_before_the_next_ending() {
    let mut timeline = vec![
        measure(0, vec![nav(JumpKind::RepeatOpen)]),
        measure(1, vec![repeat_close(2)]),
        measure(2, vec![]),
    ];
    timeline[1].barline_left = Some(Barline {
        style: BarlineStyle::Regular,
        ending: Some(Ending {
            numbers: vec![1],
            text: None,
            kind: EndingKind::Start,
        }),
    });
    set_ending(&mut timeline, 2, 2, &[2], None, true);
    let spans: Vec<_> = ending_spans(&timeline)
        .into_iter()
        .map(|span| (span.first, span.last, span.closed))
        .collect();
    assert_eq!(spans, vec![(1, 1, true), (2, 2, true)]);
}

/// An end without a start for other passes than the open ending's is an
/// ending of its own; the open one stops before it.
#[test]
fn an_unpaired_end_for_other_passes_is_its_own_ending() {
    let mut timeline = vec![measure(0, vec![]), measure(1, vec![])];
    let barline = |kind, numbers: Vec<u8>| {
        Some(Barline {
            style: BarlineStyle::Regular,
            ending: Some(Ending {
                numbers,
                text: None,
                kind,
            }),
        })
    };
    timeline[0].barline_left = barline(EndingKind::Start, vec![1]);
    timeline[1].barline_right = barline(EndingKind::Stop, vec![2]);
    let spans: Vec<_> = ending_spans(&timeline)
        .into_iter()
        .map(|span| (span.first, span.last, span.numbers.to_vec(), span.closed))
        .collect();
    assert_eq!(spans, vec![(0, 0, vec![1], false), (1, 1, vec![2], true)]);
}

/// A coda sign printed both where the al Coda pass branches off and at the
/// coda: the pass jumps from the first to the second.
#[test]
fn a_return_pass_branches_from_one_coda_sign_to_the_next() {
    let timeline = vec![
        measure(0, vec![]),
        measure(1, vec![nav(JumpKind::Coda)]),
        measure(2, vec![]),
        measure(3, vec![nav(JumpKind::DaCapoAlCoda)]),
        measure(4, vec![nav(JumpKind::Coda)]),
    ];
    assert_eq!(order(&timeline), vec![0, 1, 2, 3, 0, 1, 4]);
}

/// Old `volta`s are read in their score's format: from MusicXML (which
/// marked a last ending, `volta_last`) as ending numbers, a start tagged
/// on a repeat's close too; from GP as pass bitmasks.
#[test]
fn a_saved_scores_voltas_are_read_each_in_its_format() -> TestResult {
    let event = |kind: &str, count: Option<u8>, volta: Option<u8>, last: bool| {
        serde_json::json!({
            "measure_index": 0, "kind": kind, "repeat_count": count,
            "volta": volta, "volta_last": last,
        })
    };
    let measure = |index: u16, navigation: Vec<serde_json::Value>| {
        serde_json::json!({
            "index": index, "tempo": null, "time_signature": null,
            "key_signature": null, "marker": null, "navigation": navigation,
            "tick_resolution": 960, "duration_ticks": 3840,
            "barline_left": null, "barline_right": null,
        })
    };
    #[derive(serde::Deserialize)]
    struct Saved {
        #[serde(
            deserialize_with = "crate::model::optimized::timeline::deserialize_timeline_alone"
        )]
        timeline: Vec<MeasureDef>,
    }
    let endings = |timeline: serde_json::Value| -> TestResult<Vec<Option<Vec<u8>>>> {
        let saved: Saved = serde_json::from_value(serde_json::json!({ "timeline": timeline }))?;
        Ok(ending_numbers(&saved.timeline)
            .into_iter()
            .map(|numbers| numbers.map(<[u8]>::to_vec))
            .collect())
    };
    // GP: bitmasks (passes 1 and 2), none marked last; GP beams kept with
    // the first time signature.
    let mut first = measure(0, vec![event("RepeatOpen", None, None, false)]);
    first["time_signature"] = serde_json::json!({ "numerator": 4, "denominator": 4 });
    first["gp_beams"] = serde_json::json!([2, 2, 2, 2]);
    let gp = serde_json::json!([
        first,
        measure(1, vec![event("RepeatClose", Some(3), Some(0b011), false)]),
        measure(2, vec![event("RepeatClose", None, Some(0b100), false)]),
    ]);
    assert_eq!(endings(gp)?, vec![None, Some(vec![1, 2]), Some(vec![3])]);
    // MusicXML: ending 2 started on a repeat's close (not marked last) is
    // ending 2, not a bitmask; ending 3 is the last.
    let musicxml = serde_json::json!([
        measure(0, vec![event("RepeatOpen", None, None, false)]),
        measure(1, vec![event("RepeatClose", Some(3), Some(2), false)]),
        measure(2, vec![event("RepeatClose", None, Some(3), true)]),
    ]);
    assert_eq!(endings(musicxml)?, vec![None, Some(vec![2]), Some(vec![3])]);
    // MusicXML without a last ending marked (no GP beams either): still
    // ending numbers, ending 3 not passes 1 and 2.
    let unmarked = serde_json::json!([
        measure(0, vec![event("RepeatOpen", None, None, false)]),
        measure(1, vec![event("RepeatClose", Some(3), Some(3), false)]),
    ]);
    assert_eq!(endings(unmarked)?, vec![None, Some(vec![3])]);
    // MusicXML that kept an ending's start (on a styled barline) but not
    // its stop: the ending stops at its last measure (`volta_last`), not
    // at the next repeat barline.
    let mut start = measure(1, vec![]);
    start["barline_left"] = serde_json::json!({
        "style": "LightHeavy",
        "ending": { "numbers": [2], "text": null, "kind": "Start" },
    });
    let kept_start = serde_json::json!([
        measure(0, vec![event("RepeatOpen", None, None, false)]),
        start,
        measure(2, vec![event("RepeatClose", None, Some(2), true)]),
        measure(3, vec![]),
        measure(4, vec![event("RepeatClose", Some(2), None, false)]),
    ]);
    assert_eq!(
        endings(kept_start)?,
        vec![None, Some(vec![2]), Some(vec![2]), None, None]
    );
    Ok(())
}

#[test]
fn empty_timeline() {
    assert!(Arrangement::resolve(&[]).order.is_empty());
}

#[test]
fn linear_no_navigation() {
    let timeline: Vec<_> = (0u16..5).map(|i| measure(i, vec![])).collect();
    let order = Arrangement::resolve(&timeline).order;
    assert_eq!(order, (0u16..5).map(idx).collect::<Vec<_>>());
}

/// |: 0 | 1 | 2 :| 3   →   0 1 2  0 1 2  3
#[test]
fn simple_repeat_twice() {
    let timeline = vec![
        measure(0, vec![nav(JumpKind::RepeatOpen)]),
        measure(1, vec![]),
        measure(2, vec![repeat_close(2)]),
        measure(3, vec![]),
    ];
    assert_eq!(
        Arrangement::resolve(&timeline).order,
        vec![idx(0), idx(1), idx(2), idx(0), idx(1), idx(2), idx(3)]
    );
}

/// |: 0 | 1 :|×3   →   0 1  0 1  0 1
#[test]
fn repeat_three_times() {
    let timeline = vec![
        measure(0, vec![nav(JumpKind::RepeatOpen)]),
        measure(1, vec![repeat_close(3)]),
        measure(2, vec![]),
    ];
    assert_eq!(
        Arrangement::resolve(&timeline).order,
        vec![idx(0), idx(1), idx(0), idx(1), idx(0), idx(1), idx(2)]
    );
}

/// |: 0 | 1 [1. 2 :| [2. 3] 4
///
/// Pass 1: 0 1 2 → jump back
/// Pass 2: 0 1 (skip 2) 3 → continue
/// Then:   4
#[test]
fn volta_two_endings() {
    let timeline = vec![
        measure(0, vec![nav(JumpKind::RepeatOpen)]),
        measure(1, vec![]),
        in_ending(measure(2, vec![repeat_close(2)]), &[1]), // 1st ending
        in_ending(measure(3, vec![]), &[2]),                // 2nd (last) ending
        measure(4, vec![]),
    ];
    assert_eq!(
        Arrangement::resolve(&timeline).order,
        //        pass 1                 pass 2
        vec![idx(0), idx(1), idx(2), idx(0), idx(1), idx(3), idx(4)]
    );
}

/// |: 0 [1. 1 :| [2. 2] [3. 3] 4   (3 endings, repeat_count = 3)
#[test]
fn volta_three_endings() {
    let timeline = vec![
        measure(0, vec![nav(JumpKind::RepeatOpen)]),
        in_ending(measure(1, vec![repeat_close(3)]), &[1]),
        in_ending(measure(2, vec![repeat_close(3)]), &[2]),
        in_ending(measure(3, vec![]), &[3]),
        measure(4, vec![]),
    ];
    assert_eq!(
        Arrangement::resolve(&timeline).order,
        vec![
            idx(0),
            idx(1), // pass 1
            idx(0),
            idx(2), // pass 2
            idx(0),
            idx(3), // pass 3
            idx(4),
        ]
    );
}

/// 0 | 1 | 2 D.C. | 3
///
/// Forward: 0 1 2 → D.C. → return pass: 0 1 2 (D.C. ignored) 3 (end of piece)
#[test]
fn da_capo() {
    let timeline = vec![
        measure(0, vec![]),
        measure(1, vec![]),
        measure(2, vec![nav(JumpKind::DaCapo)]),
        measure(3, vec![]),
    ];
    assert_eq!(
        Arrangement::resolve(&timeline).order,
        vec![idx(0), idx(1), idx(2), idx(0), idx(1), idx(2), idx(3)]
    );
}

/// 0 | 1 $ | 2 | 3 D.S. | 4
///
/// Forward: 0 1 2 3 → D.S. → return from $: 1 2 3 (D.S. ignored) 4 (end of piece)
#[test]
fn dal_segno() {
    let timeline = vec![
        measure(0, vec![]),
        measure(1, vec![nav(JumpKind::Segno)]),
        measure(2, vec![]),
        measure(3, vec![nav(JumpKind::DalSegno)]),
        measure(4, vec![]),
    ];
    assert_eq!(
        Arrangement::resolve(&timeline).order,
        vec![
            idx(0),
            idx(1),
            idx(2),
            idx(3),
            idx(1),
            idx(2),
            idx(3),
            idx(4)
        ]
    );
}

/// 0 | 1 Fine | 2 | 3 D.C. al Fine
///
/// Forward: 0 1 2 3 → D.C. al Fine → 0 1 (Fine → stop)
#[test]
fn da_capo_al_fine() {
    let timeline = vec![
        measure(0, vec![]),
        measure(1, vec![nav(JumpKind::Fine)]),
        measure(2, vec![]),
        measure(3, vec![nav(JumpKind::DaCapoAlFine)]),
    ];
    assert_eq!(
        Arrangement::resolve(&timeline).order,
        vec![idx(0), idx(1), idx(2), idx(3), idx(0), idx(1)]
    );
}

/// 0 | 1 $ | 2 Fine | 3 | 4 D.S. al Fine
///
/// Forward: 0 1 2 3 4 → D.S. al Fine → 1 2 (Fine → stop)
#[test]
fn dal_segno_al_fine() {
    let timeline = vec![
        measure(0, vec![]),
        measure(1, vec![nav(JumpKind::Segno)]),
        measure(2, vec![nav(JumpKind::Fine)]),
        measure(3, vec![]),
        measure(4, vec![nav(JumpKind::DalSegnoAlFine)]),
    ];
    assert_eq!(
        Arrangement::resolve(&timeline).order,
        vec![idx(0), idx(1), idx(2), idx(3), idx(4), idx(1), idx(2)]
    );
}

/// 0 | 1 ⊕(dest) | 2 | 3 D.C. al Coda
///
/// On the return pass, measure 1's Coda has no coda sign after it to
/// branch to; execution continues from 1 to end.
/// Result: 0 1 2 3  0 1 2 3
#[test]
fn da_capo_al_coda_single_marker() {
    let timeline = vec![
        measure(0, vec![]),
        measure(1, vec![nav(JumpKind::Coda)]),
        measure(2, vec![]),
        measure(3, vec![nav(JumpKind::DaCapoAlCoda)]),
    ];
    assert_eq!(
        Arrangement::resolve(&timeline).order,
        vec![
            idx(0),
            idx(1),
            idx(2),
            idx(3),
            idx(0),
            idx(1),
            idx(2),
            idx(3)
        ]
    );
}

/// 0 | 1 ⊕ | 2 To Coda | 3 D.C. al Coda | 4 ⊕
///
/// With a To Coda, the coda sign before it (a codetta) doesn't branch:
/// the return pass plays on to the To Coda.
/// Result: 0 1 2 3  0 1 2  4
#[test]
fn da_capo_al_coda_branches_at_to_coda_past_an_earlier_coda() {
    let timeline = vec![
        measure(0, vec![]),
        measure(1, vec![nav(JumpKind::Coda)]),
        measure(2, vec![nav(JumpKind::ToCoda)]),
        measure(3, vec![nav(JumpKind::DaCapoAlCoda)]),
        measure(4, vec![nav(JumpKind::Coda)]),
    ];
    assert_eq!(
        Arrangement::resolve(&timeline).order,
        vec![
            idx(0),
            idx(1),
            idx(2),
            idx(3),
            idx(0),
            idx(1),
            idx(2),
            idx(4)
        ]
    );
}

/// 0 | 1 ⊕ | 2 D.C. al Coda | 3 ⊕ | 4 To Coda | 5 ⊕
///
/// A To Coda past the coda doesn't stop the D.C. pass branching at
/// the coda sign before it.
/// Result: 0 1 2  0 1  3 4 5
#[test]
fn da_capo_al_coda_branches_at_coda_with_a_later_to_coda() {
    let timeline = vec![
        measure(0, vec![]),
        measure(1, vec![nav(JumpKind::Coda)]),
        measure(2, vec![nav(JumpKind::DaCapoAlCoda)]),
        measure(3, vec![nav(JumpKind::Coda)]),
        measure(4, vec![nav(JumpKind::ToCoda)]),
        measure(5, vec![nav(JumpKind::Coda)]),
    ];
    assert_eq!(
        Arrangement::resolve(&timeline).order,
        vec![
            idx(0),
            idx(1),
            idx(2),
            idx(0),
            idx(1),
            idx(3),
            idx(4),
            idx(5)
        ]
    );
}

/// Malformed input: RepeatOpen with no matching RepeatClose.
/// The safety guard must terminate without hanging.
#[test]
fn safety_guard_missing_repeat_close() {
    let timeline: Vec<_> = std::iter::once(measure(0, vec![nav(JumpKind::RepeatOpen)]))
        .chain((1u16..8).map(|i| measure(i, vec![])))
        .collect();
    let order = Arrangement::resolve(&timeline).order;
    // All measures must appear at least once.
    assert!(order.contains(&idx(0)));
    assert!(order.contains(&idx(7)));
}

/// An ending left without a stop closes at the next forward repeat, also
/// when a lone ending for other passes ends later on: that end is its own.
#[test]
fn an_ending_closes_at_a_forward_repeat_before_another_endings_stop() {
    let mut timeline: Vec<_> = (0u16..9).map(|i| measure(i, vec![])).collect();
    timeline[0].navigation = vec![nav(JumpKind::RepeatOpen)];
    timeline[1].navigation = vec![repeat_close(2)];
    timeline[2].barline_left = Some(Barline {
        style: BarlineStyle::Regular,
        ending: Some(Ending {
            numbers: vec![1],
            text: None,
            kind: EndingKind::Start,
        }),
    });
    timeline[4].navigation = vec![nav(JumpKind::RepeatOpen)];
    timeline[8].barline_right = Some(Barline {
        style: BarlineStyle::Regular,
        ending: Some(Ending {
            numbers: vec![3],
            text: None,
            kind: EndingKind::Discontinue,
        }),
    });
    let spans: Vec<_> = ending_spans(&timeline)
        .into_iter()
        .map(|span| (span.first, span.last, span.numbers.to_vec()))
        .collect();
    assert_eq!(spans, vec![(2, 3, vec![1]), (8, 8, vec![3])]);
}

/// Each bar plays on its repeat's pass; the bars after a repeat, and on a
/// D.C. return (which skips repeats), on pass 1.
#[test]
fn each_bar_plays_on_its_repeats_pass() {
    let timeline = vec![
        measure(0, vec![nav(JumpKind::RepeatOpen)]),
        measure(1, vec![repeat_close(2)]),
        measure(2, vec![nav(JumpKind::DaCapo)]),
    ];
    let arrangement = Arrangement::resolve(&timeline);
    let played: Vec<(u16, u8)> = arrangement
        .order
        .iter()
        .map(|index| index.0)
        .zip(arrangement.passes.iter().copied())
        .collect();
    assert_eq!(
        played,
        vec![
            (0, 1),
            (1, 1),
            (0, 2),
            (1, 2),
            (2, 1),
            (0, 1),
            (1, 1),
            (2, 1)
        ]
    );
}

/// A repeat closed without an open one starts where it plays from: after
/// the repeat before it.
#[test]
fn a_repeat_block_without_a_start_follows_the_last_repeat() {
    use crate::analysis::repeats::repeats_analysis;
    use crate::convert::optimized::legacy::legacy_song_to_loaded_score;
    use crate::model::legacy::{headers::MeasureHeader, song::Song};

    let song = Song {
        measure_headers: (0..8)
            .map(|index| MeasureHeader {
                number: index + 1,
                repeat_open: index == 0,
                repeat_close: if index == 3 || index == 7 { 1 } else { -1 },
                ..MeasureHeader::default()
            })
            .collect(),
        ..Song::default()
    };
    let timeline = legacy_song_to_loaded_score(&song).score.timeline;
    let blocks: Vec<(usize, usize)> = repeats_analysis(&[], &timeline, None, None)
        .blocks
        .iter()
        .map(|block| (block.open_idx, block.close_idx))
        .collect();
    assert_eq!(blocks, vec![(0, 3), (4, 7)]);
}

/// Each `%` repeats the bar right before its run, each double the two.
#[test]
fn a_simile_run_repeats_the_bars_right_before_it() {
    use crate::analysis::repeats::simile_runs;
    use crate::model::legacy::{measure::Measure, song::Song, track::Track};

    let track = |marks: &[Option<&str>]| Track {
        measures: marks
            .iter()
            .map(|mark| Measure {
                simile_mark: mark.map(str::to_owned),
                ..Measure::default()
            })
            .collect(),
        ..Track::default()
    };
    let simple = Some("Simple");
    let (first, second) = (Some("FirstOfDouble"), Some("SecondOfDouble"));
    let song = Song {
        tracks: vec![
            track(&[None, None, None, None, None, simple, simple, simple]),
            track(&[None, None, None, None, first, second, first, second]),
            // A double on the second bar has one bar before it, not two.
            track(&[None, first, second]),
        ],
        ..Song::default()
    };
    let sources: Vec<(usize, usize, usize)> = simile_runs(&song, None)
        .iter()
        .map(|run| (run.run_start, run.source_start, run.source_len))
        .collect();
    assert_eq!(sources, vec![(5, 4, 1), (4, 2, 2), (1, 1, 0)]);
}

/// An unknown simile mark on the first bar has no bar before it to repeat.
#[test]
fn an_unknown_simile_mark_on_the_first_bar_has_no_source() {
    use crate::analysis::repeats::track_simile_runs;
    use crate::model::legacy::{measure::Measure, track::Track};

    let track = Track {
        measures: vec![
            Measure {
                simile_mark: Some("Triple".to_owned()),
                ..Measure::default()
            },
            Measure::default(),
        ],
        ..Track::default()
    };
    let runs: Vec<(usize, usize, usize)> = track_simile_runs(&track)
        .iter()
        .map(|run| (run.run_start, run.run_len, run.source_len))
        .collect();
    assert_eq!(runs, vec![(0, 1, 0)]);
}

/// A repeat starting in the last ending of the one before, and closing
/// past it, is its own: its passes choose its endings.
/// |: 0 |1. 1 :|2. |: 2 |1. 3 :|2. 4 | 5
#[test]
fn a_repeat_starting_in_a_last_ending_plays_its_own_endings() {
    let timeline = vec![
        measure(0, vec![nav(JumpKind::RepeatOpen)]),
        in_ending(measure(1, vec![repeat_close(2)]), &[1]),
        in_ending(measure(2, vec![nav(JumpKind::RepeatOpen)]), &[2]),
        in_ending(measure(3, vec![repeat_close(2)]), &[1]),
        in_ending(measure(4, vec![]), &[2]),
        measure(5, vec![]),
    ];
    assert_eq!(order(&timeline), vec![0, 1, 0, 2, 3, 2, 4, 5]);
}

/// Plays `timeline`'s navigation (set on Demo v5's measures) after a GP5
/// round trip, and as it was.
fn after_gp5(navigation: &[(usize, JumpKind)]) -> crate::error::GpResult<(Vec<u16>, Vec<u16>)> {
    use crate::convert::legacy::loaded_score_to_legacy_song;
    use crate::convert::optimized::legacy::legacy_song_to_loaded_score;
    use crate::model::legacy::song::Song;
    use crate::tests::common::read_file;

    let mut source = Song::default();
    source.read_gp5(&read_file("test/Demo v5.gp5".to_string()))?;
    let mut loaded = legacy_song_to_loaded_score(&source);
    for measure in loaded.score.timeline.iter_mut() {
        measure.navigation.clear();
        measure.barline_left = None;
        measure.barline_right = None;
    }
    for &(index, kind) in navigation {
        loaded.score.timeline[index].navigation.push(nav(kind));
    }
    let song = loaded_score_to_legacy_song(&loaded);
    let bytes = song.write(song.version.number, None)?;
    let mut read = Song::default();
    read.read_gp5(&bytes)?;
    let gp = legacy_song_to_loaded_score(&read);
    Ok((order(&gp.score.timeline), order(&loaded.score.timeline)))
}

/// A coda sign an al Coda pass branches from is written to GP5 as its Da
/// Coda: GP keeps one measure per sign, which for its coda is the one the
/// pass lands on.
#[test]
fn a_branching_coda_sign_survives_gp5() -> crate::error::GpResult<()> {
    // 0 | 1 ⊕ | 2 D.C. al Coda | 3 | 4 ⊕ | …
    let (gp, expected) = after_gp5(&[
        (1, JumpKind::Coda),
        (2, JumpKind::DaCapoAlCoda),
        (4, JumpKind::Coda),
    ])?;
    assert_eq!(expected[..6], [0, 1, 2, 0, 1, 4]);
    assert_eq!(gp, expected);
    Ok(())
}

/// Of a sign in several measures, GP5 keeps the one playback goes by:
/// here the second Fine, where the D.S. al Fine stops.
#[test]
fn gp5_keeps_the_fine_a_jump_stops_at() -> crate::error::GpResult<()> {
    // 0 | 1 Fine | 2 | 3 𝄋 | 4 | 5 Fine | 6 D.S. al Fine | …
    let (gp, expected) = after_gp5(&[
        (1, JumpKind::Fine),
        (3, JumpKind::Segno),
        (5, JumpKind::Fine),
        (6, JumpKind::DalSegnoAlFine),
    ])?;
    assert_eq!(expected, [0, 1, 2, 3, 4, 5, 6, 3, 4, 5]);
    assert_eq!(gp, expected);
    Ok(())
}

/// Repeat blocks and simile runs serialize with 1-based bars; a run with
/// nothing before it has no source.
#[test]
fn repeat_analysis_serializes_with_one_based_bars() -> Result<(), serde_json::Error> {
    use crate::analysis::repeats::{RepeatBlock, SimileRun};

    let block = RepeatBlock {
        open_idx: 0,
        close_idx: 3,
        total_plays: 2,
        plays: 2,
        voltas: vec![(3, vec![1])],
    };
    assert_eq!(
        serde_json::to_value(&block)?,
        serde_json::json!({
            "open_bar": 1, "close_bar": 4, "total_plays": 2, "plays": 2,
            "volta_bars": [{"bar": 4, "endings": [1]}],
        })
    );
    let run = |run_start, source_start, source_len| SimileRun {
        track_name: "Guitar".to_owned(),
        run_start,
        run_len: 2,
        source_start,
        source_len,
        kind: "1-bar (Simple)".to_owned(),
    };
    assert_eq!(
        serde_json::to_value(run(5, 4, 1))?,
        serde_json::json!({
            "track": "Guitar", "bars": "6-7", "source_bars": "5-5", "kind": "1-bar (Simple)",
        })
    );
    assert_eq!(
        serde_json::to_value(run(0, 0, 0))?,
        serde_json::json!({ "track": "Guitar", "bars": "1-2", "kind": "1-bar (Simple)" })
    );
    Ok(())
}

/// A score's timeline reads back as it was written, through the older
/// forms' reader (`SerializedMeasureDef`, mirroring `MeasureDef`): a field
/// it leaves out, or one written only when set that it doesn't default,
/// shows here.
#[test]
fn a_saved_timeline_reads_back_as_written() -> Result<(), Box<dyn std::error::Error>> {
    use crate::convert::optimized::legacy::legacy_song_to_loaded_score;
    use crate::model::legacy::song::Song;
    use crate::model::optimized::global::Score;
    use crate::tests::common::read_file;

    let mut song = Song::default();
    song.read_gp5(&read_file("test/Demo v5.gp5".to_string()))?;
    let mut score = legacy_song_to_loaded_score(&song).score;
    // One measure with every optional field set, one with none.
    let timeline = &mut score.timeline;
    timeline[0].gp_beams = Some([2, 2, 2, 2]);
    timeline[0].gp_fermatas = vec![("Medium".to_owned(), "1/1".to_owned())];
    timeline[0].gp_free_time = true;
    timeline[0].navigation = vec![nav(JumpKind::Segno)];
    timeline[1].gp_beams = None;
    timeline[1].gp_fermatas.clear();
    timeline[1].gp_free_time = false;
    timeline[1].navigation.clear();
    timeline[1].tempo = None;
    timeline[1].time_signature = None;
    timeline[1].key_signature = None;
    timeline[1].marker = None;
    timeline[1].barline_left = None;
    timeline[1].barline_right = None;

    let written = serde_json::to_value(&score)?;
    let read: Score = serde_json::from_value(written.clone())?;
    assert_eq!(serde_json::to_value(&read)?, written);
    Ok(())
}

/// A jump the model can't hold (D.S.S.) in a bar no pass plays isn't one
/// the sequence leaves out; in a bar it plays, it is.
#[test]
fn an_unfollowed_jump_in_a_bar_never_played_isnt_skipped() {
    use crate::analysis::repeats::repeats_analysis;
    use crate::model::legacy::{enums::DirectionSign, headers::MeasureHeader};

    // Fine | D.C. al Fine | (never played: the return stops at the Fine)
    let timeline = vec![
        measure(0, vec![nav(JumpKind::Fine)]),
        measure(1, vec![nav(JumpKind::DaCapoAlFine)]),
        measure(2, vec![]),
    ];
    let headers = |with_dss: usize| -> Vec<MeasureHeader> {
        (0..3)
            .map(|index| {
                let mut header = MeasureHeader::default();
                if index == with_dss {
                    header.add_directions([DirectionSign::DaSegnoSegno]);
                }
                header
            })
            .collect()
    };
    let analysis = repeats_analysis(&headers(2), &timeline, None, None);
    assert_eq!(analysis.sequence.reached, vec![true, true, false]);
    assert!(!analysis.skips_jumps);
    assert!(repeats_analysis(&headers(1), &timeline, None, None).skips_jumps);
}

/// A D.S. without a segno can't be followed: the play sequence says it
/// skips it rather than that the score has no jumps.
#[test]
fn a_dal_segno_without_a_segno_is_skipped() {
    use crate::analysis::repeats::repeats_analysis;
    use crate::convert::optimized::legacy::legacy_song_to_loaded_score;
    use crate::model::legacy::{enums::DirectionSign, headers::MeasureHeader, song::Song};

    let song = Song {
        measure_headers: (0..3)
            .map(|index| {
                let mut header = MeasureHeader {
                    number: index + 1,
                    ..MeasureHeader::default()
                };
                if index == 2 {
                    header.add_directions([DirectionSign::DaSegno]);
                }
                header
            })
            .collect(),
        ..Song::default()
    };
    let timeline = legacy_song_to_loaded_score(&song).score.timeline;
    let sequence = repeats_analysis(&[], &timeline, None, None).sequence;
    assert!(!sequence.follows_jumps);
    assert!(sequence.skips_jumps);
}

/// A D.S. on a D.C.'s return pass isn't taken by convention: one without
/// a segno there isn't a jump playback skips.
#[test]
fn a_dal_segno_on_the_return_pass_is_not_unfollowable() {
    let timeline = vec![
        measure(0, vec![]),
        measure(1, vec![nav(JumpKind::DaCapo)]),
        measure(2, vec![nav(JumpKind::DalSegno)]),
        measure(3, vec![]),
    ];
    let arrangement = Arrangement::resolve(&timeline);
    assert!(arrangement.jumped);
    assert!(!arrangement.unfollowable);
}

/// A segno listed after the D.S. in its own measure is still its target.
#[test]
fn a_segno_after_the_dal_segno_in_its_measure_is_followed() {
    let timeline = vec![
        measure(0, vec![]),
        measure(1, vec![nav(JumpKind::DalSegno), nav(JumpKind::Segno)]),
        measure(2, vec![]),
    ];
    let arrangement = Arrangement::resolve(&timeline);
    let order: Vec<u16> = arrangement.order.iter().map(|m| m.0).collect();
    assert_eq!(order, vec![0, 1, 1, 2]);
    assert!(arrangement.jumped);
    assert!(!arrangement.unfollowable);
}

/// A D.S. in a bar playback never reaches (past the Fine it stops at) isn't
/// one it skips.
#[test]
fn a_dal_segno_never_reached_is_not_unfollowable() {
    let timeline = vec![
        measure(0, vec![nav(JumpKind::Fine)]),
        measure(1, vec![nav(JumpKind::DaCapoAlFine)]),
        measure(2, vec![nav(JumpKind::DalSegno)]),
    ];
    let arrangement = Arrangement::resolve(&timeline);
    let order: Vec<u16> = arrangement.order.iter().map(|m| m.0).collect();
    assert_eq!(order, vec![0, 1, 0]);
    assert!(arrangement.jumped);
    assert!(!arrangement.unfollowable);
}

/// A saved GP score without beams is still read as GP by its other GP
/// fields: its volta of 3 is the bitmask of passes 1 and 2.
#[test]
fn a_saved_gp_scores_voltas_are_bitmasks_without_beams() -> TestResult {
    let timeline = serde_json::json!([
        {
            "index": 0, "tempo": null, "time_signature": null, "key_signature": null,
            "marker": { "label": "A", "kind": "Intro", "gp_color": 0xff0000 },
            "navigation": [{ "measure_index": 0, "kind": "RepeatOpen", "repeat_count": null }],
            "tick_resolution": 960, "duration_ticks": 3840,
            "barline_left": null, "barline_right": null,
        },
        {
            "index": 1, "tempo": null, "time_signature": null, "key_signature": null,
            "marker": null,
            "navigation": [{
                "measure_index": 1, "kind": "RepeatClose", "repeat_count": 3,
                "volta": 3, "volta_last": false,
            }],
            "tick_resolution": 960, "duration_ticks": 3840,
            "barline_left": null, "barline_right": null,
        },
    ]);
    #[derive(serde::Deserialize)]
    struct Saved {
        #[serde(
            deserialize_with = "crate::model::optimized::timeline::deserialize_timeline_alone"
        )]
        timeline: Vec<MeasureDef>,
    }
    let saved: Saved = serde_json::from_value(serde_json::json!({ "timeline": timeline }))?;
    assert_eq!(ending_numbers(&saved.timeline)[1], Some(&[1u8, 2][..]));
    Ok(())
}

/// A measure in the current form reads back as it was written, both with
/// its optional fields left out and with them all given: the serialized
/// form the older saves are read through keeps up with `MeasureDef`.
#[test]
fn measures_read_back_through_the_serialized_form() -> TestResult {
    use crate::model::optimized::timeline::{Marker, MarkerKind};

    let mut full = measure(1, vec![repeat_close(3)]);
    full.tempo = Some(90.0);
    full.marker = Some(Marker {
        label: "A".into(),
        kind: MarkerKind::Intro,
        gp_color: Some(0x00ff00),
    });
    full.barline_right = Some(Barline {
        style: BarlineStyle::LightHeavy,
        ending: None,
    });
    full.gp_beams = Some([2, 2, 2, 2]);
    full.gp_fermatas = vec![("Medium".into(), "1/1".into())];
    full.gp_free_time = true;
    let timeline = vec![measure(0, vec![nav(JumpKind::RepeatOpen)]), full];

    #[derive(serde::Deserialize)]
    struct Saved {
        #[serde(
            deserialize_with = "crate::model::optimized::timeline::deserialize_timeline_alone"
        )]
        timeline: Vec<MeasureDef>,
    }
    let json = serde_json::json!({ "timeline": &timeline });
    let saved: Saved = serde_json::from_value(json.clone())?;
    assert_eq!(serde_json::json!({ "timeline": saved.timeline }), json);
    Ok(())
}

/// A repeat close without a count plays twice, and is written so to GP
/// and MSCZ; one of more passes than GP keeps is kept at GP's most, and
/// reported.
#[test]
fn repeat_counts_are_written_as_played() -> TestResult {
    use crate::convert::legacy::{
        loaded_score_to_legacy_song, loaded_score_to_legacy_song_with_losses,
    };
    use crate::convert::mscz::loaded_score_to_mscx;
    use crate::convert::optimized::score_partwise_to_loaded_score;
    use crate::model::optimized::LoadedScore;
    use crate::tests::common::read_musicxml;

    let mut loaded = score_partwise_to_loaded_score(&read_musicxml("45a-SimpleRepeat.xml")?);
    fn close(loaded: &mut LoadedScore) -> Result<&mut NavigationEvent, &'static str> {
        loaded.score.timeline[0]
            .navigation
            .iter_mut()
            .find(|event| event.kind == JumpKind::RepeatClose)
            .ok_or("repeat close")
    }
    close(&mut loaded)?.repeat_count = None;
    assert_eq!(order(&loaded.score.timeline), vec![0, 0, 1]);
    let song = loaded_score_to_legacy_song(&loaded);
    assert_eq!(song.measure_headers[0].repeat_close, 1);
    let xml = loaded_score_to_mscx(&loaded).raw_xml;
    assert!(xml.contains("<endRepeat>2</endRepeat>"));

    close(&mut loaded)?.repeat_count = Some(200);
    let song = loaded_score_to_legacy_song(&loaded);
    assert_eq!(song.measure_headers[0].repeat_close, 126);
    assert_eq!(
        loaded_score_to_legacy_song_with_losses(&loaded)
            .1
            .get("Repeat of more than 127 passes (kept as 127)"),
        1
    );
    Ok(())
}

#[test]
fn long_repeat_plays_out_in_full() {
    // 20 measures repeated 40 times: past the old fixed guard (640).
    let mut timeline: Vec<MeasureDef> = (0..20).map(|i| measure(i, Vec::new())).collect();
    timeline[19].navigation.push(repeat_close(40));
    let arrangement = Arrangement::resolve(&timeline);
    assert_eq!(arrangement.order.len(), 800);
    assert!(!arrangement.truncated);
}

/// A repeat with a third ending plays three times without a count: its
/// block's plays say so too, as the play order plays it, its total plays
/// the written two.
#[test]
fn repeat_blocks_count_the_passes_the_order_plays() {
    use crate::analysis::repeats::repeats_analysis;

    // |: A | [1.2. B :| [3. C |
    let mut timeline = vec![
        measure(0, vec![nav(JumpKind::RepeatOpen)]),
        in_ending(measure(1, vec![nav(JumpKind::RepeatClose)]), &[1, 2]),
        in_ending(measure(2, vec![]), &[3]),
    ];
    timeline[2].barline_right = None;
    assert_eq!(order(&timeline), vec![0, 1, 0, 1, 0, 2]);
    let analysis = repeats_analysis(&[], &timeline, None, None);
    assert_eq!(analysis.blocks.len(), 1);
    // As played, and as written.
    assert_eq!(analysis.blocks[0].plays, 3);
    assert_eq!(analysis.blocks[0].total_plays, 2);
    let close = analysis.nav_events.iter().find(|event| event.bar == 2);
    assert_eq!(close.and_then(|event| event.repeat_plays), Some(3));
    assert_eq!(close.and_then(|event| event.repeat_close), Some(2));
}

/// One repeat's endings running straight into the next repeat's, opened
/// in the first's last ending: each group's last ending is its own, so a
/// D.C. return pass plays the first repeat's last ending too.
#[test]
fn adjacent_ending_groups_keep_their_own_last_ending() {
    // |: A | [1. B :| [2. |: C | [1. D :| [2. E :| [3. F | G D.C.
    let timeline = vec![
        measure(0, vec![nav(JumpKind::RepeatOpen)]),
        in_ending(measure(1, vec![nav(JumpKind::RepeatClose)]), &[1]),
        in_ending(measure(2, vec![nav(JumpKind::RepeatOpen)]), &[2]),
        in_ending(measure(3, vec![nav(JumpKind::RepeatClose)]), &[1]),
        in_ending(measure(4, vec![nav(JumpKind::RepeatClose)]), &[2]),
        in_ending(measure(5, vec![]), &[3]),
        measure(6, vec![nav(JumpKind::DaCapo)]),
    ];
    let order = order(&timeline);
    // The return pass, after G: A, the first repeat's last ending (C),
    // then the second's (F) and G.
    let return_pass = order
        .iter()
        .position(|&m| m == 6)
        .map(|at| &order[at + 1..]);
    assert_eq!(return_pass, Some(&[0, 2, 5, 6][..]));
}

/// A close in a first ending sends the repeat back on that pass only:
/// it plays twice, though its endings run on to pass 3.
#[test]
fn a_close_in_an_ending_plays_its_passes_only() {
    use crate::analysis::repeats::repeats_analysis;

    // |: A | [1. B :| [2.3. C |
    let mut timeline = vec![
        measure(0, vec![nav(JumpKind::RepeatOpen)]),
        in_ending(measure(1, vec![nav(JumpKind::RepeatClose)]), &[1]),
        in_ending(measure(2, vec![]), &[2, 3]),
    ];
    timeline[2].barline_right = None;
    assert_eq!(order(&timeline), vec![0, 1, 0, 2]);
    let analysis = repeats_analysis(&[], &timeline, None, None);
    assert_eq!(analysis.blocks[0].plays, 2);
}
