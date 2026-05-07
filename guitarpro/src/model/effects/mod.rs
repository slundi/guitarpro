mod io;
pub use io::SongEffectOps;

use crate::{
    error::{GpResult, ToPrimitiveGp},
    model::{chord::PitchClass, enums::*, key_signature::*},
};

/// A single point within the BendEffect
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BendPoint {
    pub position: u8,
    pub value: i8,
    pub vibrato: bool,
}
impl BendPoint {
    fn _get_time(&self, duration: u8) -> GpResult<u16> {
        let time =
            f32::from(duration) * f32::from(self.position) / f32::from(BEND_EFFECT_MAX_POSITION);
        Ok(time.to_i16_gp("bend point time")? as u16)
    }
}

pub const BEND_EFFECT_MAX_POSITION: u8 = 12;
pub const GP_BEND_SEMITONE: f32 = 25.0;
pub const GP_BEND_POSITION: f32 = 60.0;
pub const GP_BEND_SEMITONE_LENGTH: f32 = 1.0;

/// This effect is used to describe string bends and tremolo bars
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BendEffect {
    pub kind: BendType,
    pub value: i16,
    pub points: Vec<BendPoint>,
    pub semitone_length: u8,
    pub max_position: u8,
    pub max_value: u8,
}
impl Default for BendEffect {
    fn default() -> Self {
        BendEffect {
            kind: BendType::None,
            value: 0,
            points: Vec::with_capacity(12),
            semitone_length: 1,
            max_position: BEND_EFFECT_MAX_POSITION,
            max_value: 12,
        }
    }
}

// A collection of velocities / dynamics
pub const MIN_VELOCITY: i16 = 15;
pub const VELOCITY_INCREMENT: i16 = 16;
pub const FORTE: i16 = MIN_VELOCITY + VELOCITY_INCREMENT * 5;
pub const DEFAULT_VELOCITY: i16 = FORTE;

/// Convert Guitar Pro dynamic value to raw MIDI velocity
pub(crate) fn unpack_velocity(v: i16) -> i16 {
    MIN_VELOCITY + VELOCITY_INCREMENT * v - VELOCITY_INCREMENT
}

pub(crate) fn pack_velocity(velocity: i16) -> GpResult<i8> {
    ((velocity + VELOCITY_INCREMENT - MIN_VELOCITY) as f32 / VELOCITY_INCREMENT as f32)
        .ceil()
        .to_i8_gp("pack_velocity")
}

/// A grace note effect
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraceEffect {
    pub duration: u8,
    pub fret: i8,
    pub is_dead: bool,
    pub is_on_beat: bool,
    pub transition: GraceEffectTransition,
    pub velocity: i16,
}
impl Default for GraceEffect {
    fn default() -> Self {
        GraceEffect {
            duration: 1,
            fret: 0,
            is_dead: false,
            is_on_beat: false,
            transition: GraceEffectTransition::None,
            velocity: DEFAULT_VELOCITY,
        }
    }
}
impl GraceEffect {
    pub(crate) fn _duration_time(self) -> GpResult<i16> {
        let quarter_time =
            crate::model::key_signature::DURATION_QUARTER_TIME.to_i16_gp("quarter time")?;
        let time = f32::from(quarter_time) / 16f32 * f32::from(self.duration);
        time.to_i16_gp("grace duration time")
    }
}

/// A harmonic note effect
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarmonicEffect {
    pub kind: HarmonicType,
    pub pitch: Option<PitchClass>,
    pub octave: Option<Octave>,
    pub fret: Option<i8>,
}
impl Default for HarmonicEffect {
    fn default() -> Self {
        HarmonicEffect {
            kind: HarmonicType::Natural,
            pitch: None,
            octave: None,
            fret: None,
        }
    }
}

/// A tremolo picking effect.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TremoloPickingEffect {
    pub duration: Duration,
}

pub(crate) fn from_tremolo_value(value: i8) -> GpResult<u8> {
    match value {
        1 => Ok(DURATION_EIGHTH),
        2 => Ok(DURATION_SIXTEENTH),
        3 => Ok(DURATION_THIRTY_SECOND),
        _ => Ok(DURATION_SIXTEENTH),
    }
}

/// A trill effect.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TrillEffect {
    pub fret: i8,
    pub duration: Duration,
}

pub(crate) fn from_trill_period(period: i8) -> GpResult<u16> {
    match period {
        1 => Ok(u16::from(DURATION_SIXTEENTH)),
        2 => Ok(u16::from(DURATION_THIRTY_SECOND)),
        3 => Ok(u16::from(DURATION_SIXTY_FOURTH)),
        _ => Ok(u16::from(DURATION_SIXTEENTH)),
    }
}
