use fraction::ToPrimitive;
use std::convert::Into;

use crate::{io::*, chord::*, key_signature::*};

/// A single point within the BendEffect
#[derive(Clone,PartialEq)]
pub struct BendPoint {
    pub position: u8,
    pub value: i8,
    pub vibrato: bool,
}
impl Default for BendPoint { fn default() -> Self { BendPoint { position: 0, value: 0, vibrato: false }}}
impl BendPoint {
    /// Gets the exact time when the point need to be played (MIDI)
    /// * `duration`: the full duration of the effect
    fn get_time(&self, duration: u8) -> u16{
        return (f32::from(duration) * f32::from(self.position) / f32::from(BEND_EFFECT_MAX_POSITION)).to_i16().expect("Cannot get bend point time") as u16;
    }
}

/// All Bend presets
#[derive(Clone,PartialEq)]
pub enum BendType {
    /// No Preset.
    None,

    //Bends
    /// A simple bend.
    Bend,
    /// A bend and release afterwards.
    BendRelease,
    /// A bend, then release and rebend.
    BendReleaseBend,
    /// Prebend.
    Prebend,
    /// Prebend and then release.
    PrebendRelease,

    //Tremolo Bar
    /// Dip the bar down and then back up.
    Dip,
    /// Dive the bar.
    Dive,
    /// Release the bar up.
    ReleaseUp,
    /// Dip the bar up and then back down.
    InvertedDip,
    /// Return the bar.
    Return,
    /// Release the bar down.
    ReleaseDown
}

pub const BEND_EFFECT_MAX_POSITION: u8 =12;

pub const GP_BEND_SEMITONE: f32 = 25.0;
pub const GP_BEND_POSITION: f32 = 60.0;
/// This effect is used to describe string bends and tremolo bars
#[derive(Clone, PartialEq)]
pub struct BendEffect {
    pub kind: BendType,
    pub value: i16,
    pub points: Vec<BendPoint>,
    /// The note offset per bend point offset
    pub semitone_length: u8,
    /// The max position of the bend points (x axis)
    pub max_position: u8,
    /// The max value of the bend points (y axis)
    pub max_value: u8,
}
impl Default for BendEffect { fn default() -> Self { BendEffect { kind: BendType::None, value: 0, points: Vec::with_capacity(12), semitone_length: 1, max_position: BEND_EFFECT_MAX_POSITION, max_value: 12 /* semi_tone_length * 12 */ }}}
impl BendEffect {
    /// Read a bend. It is encoded as:
    /// - Bend type: `signed-byte`. See BendType.
    /// - Bend value: `int`.
    /// - Number of bend points: `int`.
    /// - List of points. Each point consists of:
    ///   * Position: `int`. Shows where point is set along *x*-axis.
    ///   * Value: `int`. Shows where point is set along *y*-axis.
    ///   * Vibrato: `bool`.
    pub fn read(data: &Vec<u8>, seek: &mut usize) -> Option<BendEffect> {
        let mut be = BendEffect::default();
        be.kind = match read_signed_byte(data, seek) {
            0 => BendType::None,
            1 => BendType::Bend,
            2 => BendType::BendRelease,
            3 => BendType::BendReleaseBend,
            4 => BendType::Prebend,
            5 => BendType::PrebendRelease,
            6 => BendType::Dip,
            7 => BendType::Dive,
            8 => BendType::ReleaseUp,
            9 => BendType::InvertedDip,
            10 => BendType::Return,
            11 => BendType::ReleaseDown,
            _ => panic!("Cannot read bend type"),
        };
        be.value = read_int(data, seek).to_i16().unwrap();
        let count: u8 = read_int(data, seek).try_into().unwrap();
        for _ in 0..count {
            let mut bp = BendPoint::default();
            bp.position = (f32::from(read_int(data, seek).to_i16().unwrap()) * f32::from(BEND_EFFECT_MAX_POSITION) / GP_BEND_POSITION).round().to_u8().unwrap();
            bp.value = (f32::from(read_int(data, seek).to_i16().unwrap()) * f32::from(be.semitone_length) / GP_BEND_SEMITONE).round().to_i8().unwrap();
            bp.vibrato = read_bool(data, seek);
            be.points.push(bp);
        }
        if count > 0 {return Some(be);}
        else {return None;}
    }
}

/// All transition types for grace notes.
#[derive(Clone,PartialEq)]
pub enum GraceEffectTransition {
    ///No transition
    None,
    ///Slide from the grace note to the real one.
    Slide,
    ///Perform a bend from the grace note to the real one.
    Bend,
    ///Perform a hammer on.
    Hammer
}

//A collection of velocities / dynamics
pub const MIN_VELOCITY: u16 = 15;
pub const VELOCITY_INCREMENT: u16 = 16;
pub const PIANO_PIANISSIMO: u16 = MIN_VELOCITY * VELOCITY_INCREMENT;
pub const PIANO: u16 = MIN_VELOCITY + VELOCITY_INCREMENT * 2;
pub const MEZZO_PIANO: u16 = MIN_VELOCITY + VELOCITY_INCREMENT * 3;
pub const MEZZO_FORTE: u16 = MIN_VELOCITY + VELOCITY_INCREMENT * 4;
pub const FORTE: u16 = MIN_VELOCITY + VELOCITY_INCREMENT * 5;
pub const FORTISSIMO: u16 = MIN_VELOCITY + VELOCITY_INCREMENT * 6;
pub const FORTE_FORTISSIMO: u16 = MIN_VELOCITY + VELOCITY_INCREMENT * 7;
pub const DEFAULT_VELOCITY: u16 = FORTE;
/// Convert Guitar Pro dynamic value to raw MIDI velocity
fn unpack_velocity(v: u16) -> u16 {
    return MIN_VELOCITY + VELOCITY_INCREMENT * v - VELOCITY_INCREMENT;
}

/// A grace note effect
#[derive(Clone, PartialEq)]
pub struct GraceEffect {
    pub duration: u8,
    pub fret: i8,
    pub is_dead: bool,
    pub is_on_beat: bool,
    pub transition: GraceEffectTransition,
    pub velocity: u16,
}
impl Default for GraceEffect { fn default() -> Self { GraceEffect {duration: 1, fret: 0, is_dead: false, is_on_beat: false, transition: GraceEffectTransition::None, velocity: DEFAULT_VELOCITY }}}
impl GraceEffect {
    pub fn duration_time(self) -> u16 {
        return (f32::from(crate::key_signature::DURATION_QUARTER_TIME as i16) / 16f32 * f32::from(self.duration)).to_i16().expect("Cannot get bend point time") as u16;
    }

    /// Read grace note effect.
    /// 
    /// - Fret: `signed-byte`. Number of fret.
    /// - Dynamic: `byte`. Dynamic of a grace note.
    /// - Transition: `byte`. See GraceEffectTransition`.
    /// - Duration: `byte`. Values are:
    ///   - *1*: Thirty-second note.
    ///   - *2*: Twenty-fourth note.
    ///   - *3*: Sixteenth note.
    pub fn read(data: &Vec<u8>, seek: &mut usize) -> GraceEffect {
        let mut g = GraceEffect::default();
        g.fret = read_signed_byte(data, seek);
        g.velocity = unpack_velocity(read_byte(data, seek).into());
        g.duration = 1 << (7 - read_byte(data, seek));
        g.is_dead = g.fret == -1;
        g.transition = match read_signed_byte(data, seek) {
            0 => GraceEffectTransition::None,
            1 => GraceEffectTransition::Slide,
            2 => GraceEffectTransition::Bend,
            3 => GraceEffectTransition::Hammer,
            _ => panic!("Cannot get transition for the grace effect"),
        };
        return g;
    }
}

#[derive(Clone,PartialEq)]
pub enum HarmonicType {
    Natural, //1
    Artificial,
    Tapped,
    Pinch,
    Semi, //5
}

/// A harmonic note effect
#[derive(Clone,PartialEq)]
pub struct HarmonicEffect {
    pub kind: HarmonicType,
    //artificial harmonic
    pub pitch: Option<PitchClass>,
    pub octave:Option<i8>,
    //tapped harmonic
    pub fret: Option<i8>,
}
impl Default for HarmonicEffect { fn default() -> Self {HarmonicEffect { kind: HarmonicType::Natural, pitch: None, octave: None, fret: None}}}

/// A tremolo picking effect.
#[derive(Clone,PartialEq)]
pub struct TremoloPickingEffect {duration: Duration,}
impl Default for TremoloPickingEffect { fn default() -> Self {TremoloPickingEffect { duration: Duration::default() }}}

/// A trill effect.
#[derive(Clone,PartialEq)]
pub struct TrillEffect {
    fret: i8,
    duration: Duration,
}
impl Default for TrillEffect { fn default() -> Self {TrillEffect { fret:0, duration: Duration::default() }}}
