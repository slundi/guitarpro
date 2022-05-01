use fraction::ToPrimitive;

use crate::{io::*, gp::*, chord::*, key_signature::*, enums::*};

/// A single point within the BendEffect
#[derive(Debug,Clone,PartialEq,Default)]
pub struct BendPoint {
    pub position: u8,
    pub value: i8,
    pub vibrato: bool,
}
//impl Default for BendPoint { fn default() -> Self { BendPoint { position: 0, value: 0, vibrato: false }}}
impl BendPoint {
    /// Gets the exact time when the point need to be played (MIDI)
    /// * `duration`: the full duration of the effect
    fn get_time(&self, duration: u8) -> u16{
        (f32::from(duration) * f32::from(self.position) / f32::from(BEND_EFFECT_MAX_POSITION)).to_i16().expect("Cannot get bend point time") as u16
    }
}

pub const BEND_EFFECT_MAX_POSITION: u8 =12;

pub const GP_BEND_SEMITONE: f32 = 25.0;
pub const GP_BEND_POSITION: f32 = 60.0;
pub const GP_BEND_SEMITONE_LENGTH: f32 = 1.0;
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
//pub const PIANO_PIANISSIMO: i16 = MIN_VELOCITY * VELOCITY_INCREMENT;
//pub const PIANO: i16 = MIN_VELOCITY + VELOCITY_INCREMENT * 2;
//pub const MEZZO_PIANO: i16 = MIN_VELOCITY + VELOCITY_INCREMENT * 3;
//pub const MEZZO_FORTE: i16 = MIN_VELOCITY + VELOCITY_INCREMENT * 4;
pub const FORTE: i16 = MIN_VELOCITY + VELOCITY_INCREMENT * 5;
//pub const FORTISSIMO: i16 = MIN_VELOCITY + VELOCITY_INCREMENT * 6;
//pub const FORTE_FORTISSIMO: i16 = MIN_VELOCITY + VELOCITY_INCREMENT * 7;
pub const DEFAULT_VELOCITY: i16 = FORTE;
/// Convert Guitar Pro dynamic value to raw MIDI velocity
pub(crate) fn unpack_velocity(v: i16) -> i16 {
    //println!("unpack_velocity({})", v);
    MIN_VELOCITY + VELOCITY_INCREMENT * v - VELOCITY_INCREMENT
}

pub(crate) fn pack_velocity(velocity: i16) -> i8 { ((velocity + VELOCITY_INCREMENT - MIN_VELOCITY).to_f32().unwrap()/ VELOCITY_INCREMENT.to_f32().unwrap()).ceil().to_i8().unwrap() }

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
    pub(crate) fn duration_time(self) -> i16 {
        (f32::from(crate::key_signature::DURATION_QUARTER_TIME.to_i16().unwrap()) / 16f32 * f32::from(self.duration)).to_i16().expect("Cannot get bend point time").to_i16().unwrap()
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
#[derive(Debug,Clone,PartialEq,Default)]
pub struct TremoloPickingEffect {pub duration: Duration,}
//impl Default for TremoloPickingEffect { fn default() -> Self {TremoloPickingEffect { duration: Duration::default() }}}
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
#[derive(Debug,Clone,PartialEq,Default)]
pub struct TrillEffect {
    pub fret: i8,
    pub duration: Duration,
}
//impl Default for TrillEffect { fn default() -> Self {TrillEffect { fret:0, duration: Duration::default() }}}

impl Song {
    /// Read a bend. It is encoded as:
    /// - Bend type: `signed-byte`. See BendType.
    /// - Bend value: `int`.
    /// - Number of bend points: `int`.
    /// - List of points. Each point consists of:
    ///   * Position: `int`. Shows where point is set along *x*-axis.
    ///   * Value: `int`. Shows where point is set along *y*-axis.
    ///   * Vibrato: `bool`.
    pub(crate) fn read_bend_effect(&self, data: &[u8], seek: &mut usize) -> Option<BendEffect> {
        let mut be = BendEffect{kind: get_bend_type(read_signed_byte(data, seek)), ..Default::default()};
        be.value = read_int(data, seek).to_i16().unwrap();
        let count: u8 = read_int(data, seek).to_u8().unwrap();
        for _ in 0..count {
            let mut bp = BendPoint{position: (f32::from(read_int(data, seek).to_i16().unwrap()) * f32::from(BEND_EFFECT_MAX_POSITION) / GP_BEND_POSITION).round().to_u8().unwrap(), ..Default::default()};
            bp.value = (f32::from(read_int(data, seek).to_i16().unwrap()) * f32::from(be.semitone_length) / GP_BEND_SEMITONE).round().to_i8().unwrap();
            bp.vibrato = read_bool(data, seek);
            be.points.push(bp);
        }
        //println!("read_bend_effect(): {:?}", be);
        if count > 0 {Some(be)} else {None}
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
    pub(crate) fn read_grace_effect(&self, data: &[u8], seek: &mut usize) -> GraceEffect {
        //println!("read_grace_effect()");
        let mut g = GraceEffect{fret: read_signed_byte(data, seek), ..Default::default()};
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
        g
    }

    /// Read grace note effect.
    /// - Fret: `signed-byte`. Number of fret.
    /// - Dynamic: `byte`. Dynamic of a grace note, as in `Note.velocity`.
    /// - Transition: `byte`. See `GraceEffectTransition`.
    /// - Duration: `byte`. Values are:
    ///   - *1*: Thirty-second note.
    ///   - *2*: Twenty-fourth note.
    ///   - *3*: Sixteenth note.
    /// - Flags: `byte`.
    ///   - *0x01*: grace note is muted (dead)
    ///   - *0x02*: grace note is on beat
    pub(crate) fn read_grace_effect_v5(&self, data: &[u8], seek: &mut usize) -> GraceEffect {
        let mut g = GraceEffect{fret: read_byte(data, seek).to_i8().unwrap(), ..Default::default()};
        g.velocity = unpack_velocity(read_byte(data, seek).to_i16().unwrap());
        g.transition = get_grace_effect_transition(read_byte(data, seek).to_i8().unwrap());
        g.duration = 1 << (7 - read_byte(data, seek));
        let flags = read_byte(data, seek);
        g.is_dead = (flags &0x01) == 0x01;
        g.is_on_beat = (flags &0x02) == 0x02;
        g
    }

    /// Read tremolo picking. Tremolo constists of picking speed encoded in `signed-byte`. For value mapping refer to `from_tremolo_value()`.
    pub(crate) fn read_tremolo_picking(&self, data: &[u8], seek: &mut usize) -> TremoloPickingEffect {
        let mut tp = TremoloPickingEffect::default();
        tp.duration.value = from_tremolo_value(read_signed_byte(data, seek)).to_u16().unwrap();
        tp
    }
    ///// Read slides. Slide is encoded in `signed-byte`. See `SlideType` for value mapping.
    //pub(crate) fn read_slides(&self, data: &[u8], seek: &mut usize) -> SlideType { get_slide_type(read_signed_byte(data, seek)) }

    /// Read slides. First `byte` stores slide types:
    /// - *0x01*: shift slide
    /// - *0x02*: legato slide
    /// - *0x04*: slide out downwards
    /// - *0x08*: slide out upwards
    /// - *0x10*: slide into from below
    /// - *0x20*: slide into from above
    pub(crate) fn read_slides_v5(&self, data: &[u8], seek: &mut usize) -> Vec<SlideType> {
        let t = read_byte(data, seek);
        let mut v: Vec<SlideType> = Vec::with_capacity(6);
        if (t & 0x01) == 0x01 {v.push(SlideType::ShiftSlideTo);}
        if (t & 0x02) == 0x02 {v.push(SlideType::LegatoSlideTo);}
        if (t & 0x04) == 0x04 {v.push(SlideType::OutDownwards);}
        if (t & 0x08) == 0x08 {v.push(SlideType::OutUpWards);}
        if (t & 0x10) == 0x10 {v.push(SlideType::IntoFromBelow);}
        if (t & 0x20) == 0x20 {v.push(SlideType::IntoFromAbove);}
        v
    }
    /// Read harmonic. Harmonic is encoded in `signed-byte`. Values correspond to:
    /// - *1*: natural harmonic
    /// - *3*: tapped harmonic
    /// - *4*: pinch harmonic
    /// - *5*: semi-harmonic
    /// - *15*: artificial harmonic on (*n + 5*)th fret
    /// - *17*: artificial harmonic on (*n + 7*)th fret
    /// - *22*: artificial harmonic on (*n + 12*)th fret
    pub(crate) fn read_harmonic(&self, data: &[u8], seek: &mut usize, note: &crate::note::Note) -> HarmonicEffect {
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
                he.pitch = Some(PitchClass::from(note.real_value(&self.tracks[self.current_track.expect("Current track not defined")].strings), None, None));
                he.octave = Some(Octave::Quindicesima);
                he.kind = HarmonicType::Artificial;
            },
            22 => {
                he.pitch = Some(PitchClass::from(note.real_value(&self.tracks[self.current_track.expect("Current track not defined")].strings), None, None));
                he.octave = Some(Octave::Ottava);
                he.kind = HarmonicType::Artificial;
            },
            _ => panic!("Cannot read harmonic type"),
        };
        he
    }

    /// Read harmonic. First `byte` is harmonic type:
    /// - *1*: natural harmonic
    /// - *2*: artificial harmonic
    /// - *3*: tapped harmonic
    /// - *4*: pinch harmonic
    /// - *5*: semi-harmonic
    /// 
    /// In case harmonic types is artificial, following data is read:
    /// - Note: `byte`.
    /// - Accidental: `signed-byte`.
    /// - Octave: `byte`.
    /// 
    /// If harmonic type is tapped:
    /// - Fret: `byte`.
    pub(crate) fn read_harmonic_v5(&mut self, data: &[u8], seek: &mut usize, note: &mut crate::note::Note) -> HarmonicEffect {
        let mut he = HarmonicEffect::default();
        match read_signed_byte(data, seek) {
            1 => he.kind = HarmonicType::Natural,
            2 => {
                // C = 0, D = 2, E = 4, F = 5...
                // b = -1, # = 1
                // loco = 0, 8va = 1, 15ma = 2
                he.kind = HarmonicType::Artificial;
                let semitone = read_byte(data, seek).to_i8().unwrap();
                let accidental = read_signed_byte(data, seek);
                he.pitch = Some(PitchClass::from(semitone, Some(accidental), None));
                he.octave = Some(get_octave(read_byte(data, seek)));
            },
            3 => {
                he.kind = HarmonicType::Tapped;
                he.fret = Some(read_byte(data, seek).to_i8().unwrap());
            },
            4 => he.kind = HarmonicType::Pinch,
            5 => he.kind = HarmonicType::Semi,
            _ => panic!("Cannot read harmonic type"),
        };
        he
    }
    /// Read trill.
    /// - Fret: `signed-byte`.
    /// - Period: `signed-byte`. See `from_trill_period`.
    pub(crate) fn read_trill(&self, data: &[u8], seek: &mut usize) -> TrillEffect {
        let mut t = TrillEffect{fret: read_signed_byte(data, seek), ..Default::default()};
        t.duration.value = self.from_trill_period(read_signed_byte(data, seek));
        t
    }
    fn from_trill_period(&self, period: i8) -> u16 {
        match period {
            1 => DURATION_SIXTEENTH,
            2 => DURATION_THIRTY_SECOND,
            3 => DURATION_SIXTY_FOURTH,
            _ => panic!("Cannot get trill period"),
        }.to_u16().unwrap()
    }

    pub(crate) fn write_bend(&self, data: &mut Vec<u8>, bend: &Option<BendEffect>) {
        if let Some(b) = bend {
            write_signed_byte(data, from_bend_type(b.kind));
            write_i32(data, b.value.to_i32().unwrap());
            write_i32(data, b.points.len().to_i32().unwrap());
            for i in 0..b.points.len() {
                write_i32(data, (b.points[i].position.to_f32().unwrap() * GP_BEND_POSITION / BEND_EFFECT_MAX_POSITION.to_f32().unwrap()).round().to_i32().unwrap());
                write_i32(data, (b.points[i].value.to_f32().unwrap() * GP_BEND_SEMITONE / GP_BEND_SEMITONE_LENGTH).round().to_i32().unwrap());
                write_bool(data, b.points[i].vibrato);
            }
        }
    }
    pub(crate) fn write_grace(&self, data: &mut Vec<u8>, grace: &Option<GraceEffect>) {
        let g = grace.clone().unwrap();
        write_signed_byte(data, g.fret);
        write_byte(data, pack_velocity(g.velocity).to_u8().unwrap());
        write_byte(data, g.duration.leading_zeros().to_u8().unwrap()); //8 - grace.duration.bit_length()
        write_signed_byte(data, from_grace_effect_transition(g.transition));
    }
    pub(crate) fn write_harmonic(&self, data: &mut Vec<u8>, note: &crate::note::Note, strings: &Vec<(i8,i8)>) {
        if let Some(h) = &note.effect.harmonic {
            let mut byte = from_harmonic_type(h.kind);
            if h.kind != HarmonicType::Artificial {
                if h.pitch.is_some() && h.octave.is_some() {
                    let p = h.pitch.clone().unwrap();
                    let o = h.octave.clone().unwrap();
                    if      p.value == ((note.real_value(strings) +7) % 12) && o == Octave::Ottava {byte = 15;}
                    else if p.value == (note.real_value(strings) % 12) && o == Octave::Quindicesima {byte = 17;}
                    else if p.value == (note.real_value(strings) % 12) && o == Octave::Ottava {byte = 22;}
                } else {byte = 22;}
            }
            write_signed_byte(data, byte);
        }
    }
}
