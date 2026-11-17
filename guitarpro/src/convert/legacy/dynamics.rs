//! Dynamics as Guitar Pro's per-note velocities, for both directions.
//!
//! GP gives every note a velocity; the model marks a dynamic only where the
//! level changes. [`DynamicLevels`] holds the rules for which level is in
//! effect: the GP import ([`keep_dynamic_changes`]) marks the changes the
//! GP export ([`measure_velocities`]) reads back with it.

use std::collections::{BTreeMap, HashMap};

use crate::model::{
    legacy::effects::{DEFAULT_VELOCITY, level_velocity, nearest_dynamic_level},
    optimized::{beat::Dynamic, global::MeasureIndex, track::MeasureData},
};

/// The marking of the level nearest a velocity (GP's levels, ppp to fff),
/// as GPIF writes it ([`nearest_dynamic_level`]).
pub(crate) fn velocity_to_dynamic(velocity: i16) -> Dynamic {
    GP_LEVELS[nearest_dynamic_level(velocity)].clone()
}

/// GP's eight levels as markings, quietest first (as
/// [`DYNAMIC_LEVELS`](crate::model::legacy::effects::DYNAMIC_LEVELS) names them).
const GP_LEVELS: [Dynamic; 8] = [
    Dynamic::PPP,
    Dynamic::PP,
    Dynamic::P,
    Dynamic::MP,
    Dynamic::MF,
    Dynamic::F,
    Dynamic::FF,
    Dynamic::FFF,
];

/// The velocity a marking sounds at; `None` for one without a level: the
/// GP level nearest MuseScore's playback velocity for it
/// ([`Dynamic::playback_velocity`]), so both formats give a marking the same
/// loudness (markings beyond the eight levels take the nearest one's).
/// MuseScore gives two none: niente takes the softest level, and sfzp's
/// attack sounds as sfz's.
pub(crate) fn dynamic_to_velocity(dynamic: &Dynamic) -> Option<i16> {
    let velocity = match dynamic {
        Dynamic::N => return Some(level_velocity(0)),
        Dynamic::SFZP => Dynamic::SFZ.playback_velocity(),
        _ => dynamic.playback_velocity(),
    }?;
    Some(level_velocity(nearest_dynamic_level(i16::from(velocity))))
}

/// The level that continues after `dynamic`: accents (sfz, fz, …) return
/// to the level before them, and fp-style markings drop to their second part.
fn level_after(dynamic: &Dynamic, level: i16) -> i16 {
    match dynamic {
        Dynamic::SF
        | Dynamic::SFZ
        | Dynamic::SFF
        | Dynamic::SFFZ
        | Dynamic::RF
        | Dynamic::RFZ
        | Dynamic::FZ
        | Dynamic::Other(_) => level,
        Dynamic::FP | Dynamic::SFP | Dynamic::SFZP => level_velocity(2),
        Dynamic::SFPP => level_velocity(1),
        Dynamic::PF => level_velocity(5),
        _ => dynamic_to_velocity(dynamic).unwrap_or(level),
    }
}

/// The dynamic levels in effect (as velocities), from GP's default forte.
/// A voice follows its own last marking; one without any yet follows the
/// staff's (the last marking in any voice), so a MuseScore dynamic holds
/// for the whole staff while GP voices keep their own velocities.
pub(crate) struct DynamicLevels {
    staff: i16,
    voices: HashMap<u8, i16>,
}

impl DynamicLevels {
    pub(crate) fn new() -> Self {
        Self {
            staff: DEFAULT_VELOCITY,
            voices: HashMap::new(),
        }
    }

    /// The level `voice` plays at.
    fn level(&self, voice: u8) -> i16 {
        self.voices.get(&voice).copied().unwrap_or(self.staff)
    }

    /// Takes in a marking in `voice` (markings come in (tick, voice) order).
    pub(crate) fn apply(&mut self, voice: u8, dynamic: &Dynamic) {
        self.voices
            .insert(voice, level_after(dynamic, self.level(voice)));
        self.staff = level_after(dynamic, self.staff);
    }
}

/// The velocity of each beat of a measure, by (voice, beat index): its
/// own marking's (an unknown one's: another voice's at the same tick),
/// else its voice's level. Takes the beats in (tick, voice) order, moving
/// `levels` on past each marking.
pub(crate) fn measure_velocities(
    data: &MeasureData,
    levels: &mut DynamicLevels,
) -> HashMap<(u8, usize), i16> {
    let beats = data.beats_in_time_order();
    // The first known marking's velocity at each tick.
    let mut at_tick: HashMap<u32, i16> = HashMap::new();
    for &(_, _, beat) in &beats {
        if let Some(velocity) = beat.dynamic.as_ref().and_then(dynamic_to_velocity) {
            at_tick.entry(beat.tick_offset).or_insert(velocity);
        }
    }
    beats
        .into_iter()
        .map(|(voice, index, beat)| {
            let level = levels.level(voice);
            let velocity = match &beat.dynamic {
                Some(dynamic) => {
                    levels.apply(voice, dynamic);
                    dynamic_to_velocity(dynamic)
                        .or_else(|| at_tick.get(&beat.tick_offset).copied())
                        .unwrap_or(level)
                }
                None => level,
            };
            ((voice, index), velocity)
        })
        .collect()
}

/// Keeps only the beat dynamics (one per GP beat, from its velocity) the
/// export needs, under [`DynamicLevels`], to give the beats their
/// velocities back: those changing their voice's level. A voice's first
/// is kept too, so formats with a default of their own (MuseScore's is mf)
/// and dynamics per voice (as the MSCZ writer writes those in several
/// voices) start each voice at its level, also in a track that never
/// leaves GP's default forte. Beats are taken in (tick, voice) order.
pub(crate) fn keep_dynamic_changes(measures: &mut BTreeMap<MeasureIndex, MeasureData>) {
    let mut levels = DynamicLevels::new();
    for data in measures.values_mut() {
        for (voice, beat) in data.beats_in_time_order_mut() {
            let Some(dynamic) = beat.dynamic.take() else {
                continue;
            };
            let first = !levels.voices.contains_key(&voice);
            if first || dynamic_to_velocity(&dynamic) != Some(levels.level(voice)) {
                levels.apply(voice, &dynamic);
                beat.dynamic = Some(dynamic);
            }
        }
    }
}
