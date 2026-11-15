//! GP's direction signs (D.C., Segno, Fine, …) as the jumps of the
//! optimized model's navigation. Shared by the GP conversions both ways
//! and the loading of scores saved before the navigation had the signs.

use crate::model::{
    legacy::enums::DirectionSign,
    optimized::{
        global::{MeasureIndex, Score},
        timeline::{JumpKind, NavigationEvent},
    },
};

/// GP direction signs and the jumps and markers they stand for. The
/// double coda and segno-segno signs have none: the model has a single
/// coda and segno, and merging them with those would send playback to the
/// wrong one. Repeats are no signs.
const DIRECTIONS: [(DirectionSign, JumpKind); 10] = [
    (DirectionSign::Segno, JumpKind::Segno),
    (DirectionSign::Coda, JumpKind::Coda),
    (DirectionSign::Fine, JumpKind::Fine),
    (DirectionSign::DaCoda, JumpKind::ToCoda),
    (DirectionSign::DaCapo, JumpKind::DaCapo),
    (DirectionSign::DaCapoAlCoda, JumpKind::DaCapoAlCoda),
    (DirectionSign::DaCapoAlFine, JumpKind::DaCapoAlFine),
    (DirectionSign::DaSegno, JumpKind::DalSegno),
    (DirectionSign::DaSegnoAlCoda, JumpKind::DalSegnoAlCoda),
    (DirectionSign::DaSegnoAlFine, JumpKind::DalSegnoAlFine),
];

pub(crate) fn jump_to_direction(kind: JumpKind) -> Option<DirectionSign> {
    DIRECTIONS
        .into_iter()
        .find_map(|(sign, jump)| (jump == kind).then_some(sign))
}

pub(crate) fn direction_to_jump(sign: &DirectionSign) -> Option<JumpKind> {
    DIRECTIONS
        .into_iter()
        .find_map(|(known, jump)| (known == *sign).then_some(jump))
}

/// Adds a measure's jumps to its navigation: signs (segno, coda) mark the
/// measure's start, before its repeat; jumps (and To Coda, Fine) are taken
/// at its end, after it.
pub(crate) fn add_direction_jumps(
    navigation: &mut Vec<NavigationEvent>,
    measure_index: MeasureIndex,
    jumps: impl IntoIterator<Item = JumpKind>,
) {
    let event = |kind| NavigationEvent {
        measure_index,
        kind,
        repeat_count: None,
    };
    let (starts, ends): (Vec<JumpKind>, Vec<JumpKind>) = jumps
        .into_iter()
        .partition(|kind| matches!(kind, JumpKind::Segno | JumpKind::Coda));
    navigation.splice(0..0, starts.into_iter().map(event));
    navigation.extend(ends.into_iter().map(event));
}

/// The `identification.miscellaneous` entry keeping measure `index`'s
/// direction signs the timeline has no jump for, by their names
/// ([`DirectionSign::name`]): `gp.direction.N`, the names joined by commas.
pub(crate) fn direction_entry<'a>(
    index: usize,
    names: impl IntoIterator<Item = &'a str>,
) -> (String, String) {
    (
        format!("{DIRECTION_KEY_PREFIX}{index}"),
        names.into_iter().collect::<Vec<_>>().join(","),
    )
}

/// A [`direction_entry`]'s measure index and sign names; `None` for
/// another entry.
pub(crate) fn parse_direction_entry<'a>(
    key: &str,
    value: &'a str,
) -> Option<(usize, impl Iterator<Item = &'a str>)> {
    let index = key.strip_prefix(DIRECTION_KEY_PREFIX)?.parse().ok()?;
    Some((
        index,
        value
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty()),
    ))
}

const DIRECTION_KEY_PREFIX: &str = "gp.direction.";

/// Scores saved before the navigation had the direction signs kept all of
/// them in `gp.direction.N`; moves those with a jump into the navigation,
/// leaving the others there for a GP export.
pub(crate) fn migrate_direction_signs(score: &mut Score) {
    let Some(identification) = score.metadata.identification.as_mut() else {
        return;
    };
    identification.miscellaneous.retain_mut(|(key, value)| {
        let Some((index, names)) = parse_direction_entry(key, value) else {
            return true;
        };
        let Some(measure) = score.timeline.get_mut(index) else {
            return true;
        };
        let mut jumps = Vec::new();
        let mut kept = Vec::new();
        for name in names {
            match DirectionSign::from_name(name).and_then(|sign| direction_to_jump(&sign)) {
                Some(kind) => {
                    if !jumps.contains(&kind) && !measure.has_navigation(kind) {
                        jumps.push(kind);
                    }
                }
                None => kept.push(name),
            }
        }
        let (_, kept) = direction_entry(index, kept);
        add_direction_jumps(&mut measure.navigation, measure.index, jumps);
        *value = kept;
        !value.is_empty()
    });
}
