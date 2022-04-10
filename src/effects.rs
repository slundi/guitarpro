use fraction::ToPrimitive;

use crate::{io::*, gp::*, chord::*, key_signature::*, enums::*};

/// A single point within the BendEffect
#[derive(Debug,Clone,PartialEq)]
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

pub const BEND_EFFECT_MAX_POSITION: u8 =12;

pub const GP_BEND_SEMITONE: f32 = 25.0;
pub const GP_BEND_POSITION: f32 = 60.0;
/// This effect is used to describe string bends and tremolo bars
#[derive(Debug,Clone, PartialEq)]
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

//A collection of velocities / dynamics
pub const MIN_VELOCITY: i16 = 15;
pub const VELOCITY_INCREMENT: i16 = 16;
pub const PIANO_PIANISSIMO: i16 = MIN_VELOCITY * VELOCITY_INCREMENT;
pub const PIANO: i16 = MIN_VELOCITY + VELOCITY_INCREMENT * 2;
pub const MEZZO_PIANO: i16 = MIN_VELOCITY + VELOCITY_INCREMENT * 3;
pub const MEZZO_FORTE: i16 = MIN_VELOCITY + VELOCITY_INCREMENT * 4;
pub const FORTE: i16 = MIN_VELOCITY + VELOCITY_INCREMENT * 5;
pub const FORTISSIMO: i16 = MIN_VELOCITY + VELOCITY_INCREMENT * 6;
pub const FORTE_FORTISSIMO: i16 = MIN_VELOCITY + VELOCITY_INCREMENT * 7;
pub const DEFAULT_VELOCITY: i16 = FORTE;
/// Convert Guitar Pro dynamic value to raw MIDI velocity
pub fn unpack_velocity(v: i16) -> i16 {
    //println!("unpack_velocity({})", v);
    return MIN_VELOCITY + VELOCITY_INCREMENT * v - VELOCITY_INCREMENT;
}

/// A grace note effect
#[derive(Debug,Clone, PartialEq)]
pub struct GraceEffect {
    pub duration: u8,
    pub fret: i8,
    pub is_dead: bool,
    pub is_on_beat: bool,
    pub transition: GraceEffectTransition,
    pub velocity: i16,
}
impl Default for GraceEffect { fn default() -> Self { GraceEffect {duration: 1, fret: 0, is_dead: false, is_on_beat: false, transition: GraceEffectTransition::None, velocity: DEFAULT_VELOCITY }}}
impl GraceEffect {
    pub fn duration_time(self) -> i16 {
        return (f32::from(crate::key_signature::DURATION_QUARTER_TIME.to_i16().unwrap()) / 16f32 * f32::from(self.duration)).to_i16().expect("Cannot get bend point time").to_i16().unwrap();
    }
}

/// A harmonic note effect
#[derive(Debug,Clone,PartialEq)]
pub struct HarmonicEffect {
    pub kind: HarmonicType,
    //artificial harmonic
    pub pitch: Option<PitchClass>,
    pub octave:Option<Octave>,
    //tapped harmonic
    pub fret: Option<i8>,
}
impl Default for HarmonicEffect { fn default() -> Self {HarmonicEffect { kind: HarmonicType::Natural, pitch: None, octave: None, fret: None}}}

/// A tremolo picking effect.
#[derive(Debug,Clone,PartialEq)]
pub struct TremoloPickingEffect {duration: Duration,}
impl Default for TremoloPickingEffect { fn default() -> Self {TremoloPickingEffect { duration: Duration::default() }}}
/// Convert tremolo picking speed to actual duration. Values are:
/// - *1*: eighth
/// - *2*: sixteenth
/// - *3*: thirtySecond
fn from_tremolo_value(value: i8) -> u8 {
    match value {
        1 => DURATION_EIGHTH,
        3 => DURATION_SIXTEENTH,
        2 => DURATION_THIRTY_SECOND,
        _ => panic!("Cannot get tremolo value")
    }
}

/// A trill effect.
#[derive(Debug,Clone,PartialEq)]
pub struct TrillEffect {
    fret: i8,
    duration: Duration,
}
impl Default for TrillEffect { fn default() -> Self {TrillEffect { fret:0, duration: Duration::default() }}}

impl Song {
    /// Read a bend. It is encoded as:
    /// - Bend type: `signed-byte`. See BendType.
    /// - Bend value: `int`.
    /// - Number of bend points: `int`.
    /// - List of points. Each point consists of:
    ///   * Position: `int`. Shows where point is set along *x*-axis.
    ///   * Value: `int`. Shows where point is set along *y*-axis.
    ///   * Vibrato: `bool`.
    pub fn read_bend_effect(&self, data: &Vec<u8>, seek: &mut usize) -> Option<BendEffect> {
        let mut be = BendEffect::default();
        be.kind = get_bend_type(read_signed_byte(data, seek));
        be.value = read_int(data, seek).to_i16().unwrap();
        let count: u8 = read_int(data, seek).to_u8().unwrap();
        for _ in 0..count {
            let mut bp = BendPoint::default();
            bp.position = (f32::from(read_int(data, seek).to_i16().unwrap()) * f32::from(BEND_EFFECT_MAX_POSITION) / GP_BEND_POSITION).round().to_u8().unwrap();
            bp.value = (f32::from(read_int(data, seek).to_i16().unwrap()) * f32::from(be.semitone_length) / GP_BEND_SEMITONE).round().to_i8().unwrap();
            bp.vibrato = read_bool(data, seek);
            be.points.push(bp);
        }
        //println!("read_bend_effect(): {:?}", be);
        if count > 0 {return Some(be);}
        else {return None;}
    }
    /// Read grace note effect.
    /// 
    /// - Fret: `signed-byte`. The fret number the grace note is made from.
    /// - Dynamic: `byte`. The grace note dynamic is coded like this (default value is 6):
    ///   * 1: ppp
    ///   * 2: pp
    ///   * 3: p
    ///   * 4: mp
    ///   * 5: mf
    ///   * 6: f
    ///   * 7: ff
    ///   * 8: fff
    /// - Transition: `byte`. This variable determines the transition type used to make the grace note: `0: None`, `1: Slide`, `2: Bend`, `3: Hammer` (defined in `GraceEffectTransition`).
    /// - Duration: `byte`. Determines the grace note duration, coded this way: `3: Sixteenth note`, `2: Twenty-fourth note`, `1: Thirty-second note`.
    pub fn read_grace_effect(&self, data: &Vec<u8>, seek: &mut usize) -> GraceEffect {
        //println!("read_grace_effect()");
        let mut g = GraceEffect::default();
        g.fret = read_signed_byte(data, seek);
        g.velocity = unpack_velocity(read_byte(data, seek).to_i16().unwrap());
        g.duration = 1 << (7 - read_byte(data, seek));
        //g.duration = 1 << (7 - read_byte(data, seek));
        /*g.duration = match read_byte(data, seek) {
            1 => DURATION_THIRTY_SECOND,
            2 => DURATION_TWENTY_FOURTH, //TODO: FIXME: ?
            3 => DURATION_SIXTEENTH,
            _ => panic!("Cannot get grace note effect duration"),
        };*/
        g.is_dead = g.fret == -1;
        g.transition = get_grace_effect_transition(read_signed_byte(data, seek));
        return g;
    }
    /// Read tremolo picking. Tremolo constists of picking speed encoded in `signed-byte`. For value mapping refer to `from_tremolo_value()`.
    pub fn read_tremolo_picking(&self, data: &Vec<u8>, seek: &mut usize) -> TremoloPickingEffect {
        let mut tp = TremoloPickingEffect::default();
        read_signed_byte(data, seek);//TODO: tp.duration = from_tremolo_value(read_signed_byte(data, seek));
        return tp;
    }
    /// Read slides. Slide is encoded in `signed-byte`. See `SlideType` for value mapping.
    fn read_slide(&self, data: &Vec<u8>, seek: &mut usize) -> SlideType {
        get_slide_type(read_signed_byte(data, seek))
    }
    /// Read harmonic. Harmonic is encoded in :ref:`signed-byte`. Values correspond to:
    /// - *1*: natural harmonic
    /// - *3*: tapped harmonic
    /// - *4*: pinch harmonic
    /// - *5*: semi-harmonic
    /// - *15*: artificial harmonic on (*n + 5*)th fret
    /// - *17*: artificial harmonic on (*n + 7*)th fret
    /// - *22*: artificial harmonic on (*n + 12*)th fret
    pub fn read_harmonic(&self, data: &Vec<u8>, seek: &mut usize, note: &crate::note::Note) -> HarmonicEffect {
        let mut he = HarmonicEffect::default();
        match read_signed_byte(data, seek) {
            1 => he.kind = HarmonicType::Natural,
            3 => he.kind = HarmonicType::Tapped,
            4 => he.kind = HarmonicType::Pinch,
            5 => he.kind = HarmonicType::Semi,
            15 => {
                he.pitch = Some(PitchClass::from(((note.value +7 ) % 12).to_i8().unwrap(), None, None));
                he.octave = Some(Octave::Ottava);
                he.kind = HarmonicType::Artificial;
            },
            17 => {
                he.pitch = Some(PitchClass::from(note.real_value(self.tracks[self.current_track.expect("Current track not defined")].strings.clone()), None, None));
                he.octave = Some(Octave::Quindicesima);
                he.kind = HarmonicType::Artificial;
            },
            22 => {
                he.pitch = Some(PitchClass::from(note.real_value(self.tracks[self.current_track.expect("Current track not defined")].strings.clone()), None, None));
                he.octave = Some(Octave::Ottava);
                he.kind = HarmonicType::Artificial;
            },
            _ => panic!("Cannot read harmonic type"),
        };
        return he;
    }
    /// Read trill.
    /// - Fret: `signed-byte`.
    /// - Period: `signed-byte`. See `from_trill_period`.
    pub fn read_trill(&self, data: &Vec<u8>, seek: &mut usize) -> TrillEffect {
        let mut t = TrillEffect::default();
        t.fret = read_signed_byte(data, seek);
        t.duration.value = self.from_trill_period(read_signed_byte(data, seek));
        return t;
    }
    fn from_trill_period(&self, period: i8) -> u16 {
        match period {
            1 => DURATION_SIXTEENTH,
            2 => DURATION_THIRTY_SECOND,
            3 => DURATION_SIXTY_FOURTH,
            _ => panic!("Cannot get trill period"),
        }.to_u16().unwrap()
    }
}
