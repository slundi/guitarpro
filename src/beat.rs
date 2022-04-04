use fraction::ToPrimitive;

use crate::{gp::*, mix_table::*, effects::*, chord::*, key_signature::*, note::*, io::*};


#[derive(Clone,PartialEq)]
pub enum BeatStatus {Empty, Normal, Rest}

#[derive(Clone,PartialEq)]
pub enum TupletBracket {None, Start, End}

/// Octave signs
#[derive(Clone,PartialEq)]
pub enum Octave { None, Ottava, Quindicesima, Ottavabassa, Quindicesimabassa }

/// A beat contains multiple notes
#[derive(Clone,PartialEq)]
pub struct Beat {
    //TODO: pub voice: Voice,
    pub notes: Vec<Note>,
    pub duration: Duration,
    pub text: String,
    pub start: Option<u16>,
    pub effect: BeatEffects,
    pub octave: Octave,
    pub display: BeatDisplay,
    pub status: BeatStatus,
}
impl Default for Beat { fn default() -> Self { Beat {
    //voice
    notes: Vec::with_capacity(12),
    duration: Duration::default(),
    text: String::new(),
    start: None,
    effect: BeatEffects::default(),
    octave: Octave::None,
    display: BeatDisplay::default(),
    status: BeatStatus::Empty,
}}}
impl Beat {
    //pub fn start_in_measure(&self) -> u16 {return self.start - self.voice.measure.start;}
    pub fn has_vibrato(&self) -> bool {
        for i in 0..self.notes.len() {if self.notes[i].effect.vibrato {return true}}
        return false;
    }
    pub fn has_harmonic(&self) -> bool {
        for i in 0..self.notes.len() {if self.notes[i].effect.is_harmonic() {return true;}}
        return false;
    }
}
/// Read beat. The first byte is the beat flags. It lists the data present in the current beat:
/// - *0x01*: dotted notes- *0x02*: presence of a chord diagram
/// - *0x04*: presence of a text
/// - *0x08*: presence of effects
/// - *0x10*: presence of a mix table change event
/// - *0x20*: the beat is a n-tuplet
/// - *0x40*: status: True if the beat is empty of if it is a rest
/// - *0x80*: *blank*
/// 
/// Flags are followed by:
/// - Status: `byte`. If flag at *0x40* is true, read one byte. If value of the byte is *0x00* then beat is empty, if value is *0x02* then the beat is rest.
/// - Beat duration: `byte`. See `Duration::read()`.
/// - Chord diagram. See `Chord::read()`.
/// - Text: `int-byte-size-string`.
/// - Beat effects. See `BeatEffects::read()`.
/// - Mix table change effect. See `MixTableChange::read()`.
pub fn read_beat(data: &Vec<u8>, seek: &mut usize) -> u8 { //start,voice
    let flags = read_byte(data, seek);
    //let beat = get_beat(voice,start);
    /*if (flags & 0x40) == 0x40 {
        beat.status = match read_byte(data, seek) {
            0 => BeatStatus::Empty,
            1 => BeatStatus::Normal,
            2 => BeatStatus::Rest,
            _ => panic!("Cannot get beat status"),
        };
    } //else { beat.status = BeatStatus::Normal;}
    let duration = Duration::read(data, seek, flags);
    if (flags & 0x02) == 0x02 {beat.effect.chord = Chord::read(voice.measure.track.strings.len());}
    if (flags & 0x04) == 0x04 {beat.text = read_byte_size_string(data, seek);}
    if (flags & 0x08) == 0x08 {
        let chord = beat.effect.chord.clone();
        beat.effect = BeatEffects::read(data, seek);
        beat.effect.chord = chord;
    }
    if (flags & 0x10) == 0x10 {
        let mtc = MixTableChange::read(data, seek, voice.measure);
        beet.effect.mix_table_change = mtc;
    }
    if beat.status == BeatStatus::Empty {return 0;} else {return duration.time();}*/0
}

/// Parameters of beat display
#[derive(Clone,PartialEq)]
pub struct BeatDisplay {
    break_beam: bool,
    force_beam: bool,
    beam_direction: VoiceDirection,
    tuple_bracket: TupletBracket,
    break_secondary: u16,
    break_secondary_tuplet: bool,
    force_bracket: bool,
}
impl Default for BeatDisplay { fn default() -> Self { BeatDisplay { break_beam:false, force_beam:false, beam_direction:VoiceDirection::None, tuple_bracket:TupletBracket::None, break_secondary:0, break_secondary_tuplet:false, force_bracket:false }}}

/// A stroke effect for beats.
#[derive(Clone,PartialEq)]
pub struct BeatStroke {
    pub direction: BeatStrokeDirection,
    pub value: u16,
}
impl Default for BeatStroke { fn default() -> Self { BeatStroke { direction: BeatStrokeDirection::None, value: 0 }}}
impl BeatStroke {
    pub fn swap_direction(&self) -> BeatStroke {
        let mut bs = BeatStroke::default();
        if self.direction == BeatStrokeDirection::Up {bs.direction = BeatStrokeDirection::Down}
        else if self.direction == BeatStrokeDirection::Down {bs.direction = BeatStrokeDirection::Up}
        return bs;
    }
}

/// This class contains all beat effects
#[derive(Clone,PartialEq)]
pub struct BeatEffects {
    pub stroke: BeatStroke,
    pub has_rasgueado: bool,
    pub pick_stroke: BeatStrokeDirection,
    pub chord: Option<Chord>,
    pub fade_in: bool,
    pub tremolo_bar: Option<BendEffect>,
    pub mix_table_change: Option<MixTableChange>,
    pub slap_effect: SlapEffect,
    pub vibrato: bool,
}
impl Default for BeatEffects { fn default() -> Self { BeatEffects {
    stroke: BeatStroke::default(),
    has_rasgueado: false,
    pick_stroke: BeatStrokeDirection::None,
    chord: None,
    fade_in: false,
    tremolo_bar: None,
    mix_table_change: None,
    slap_effect: SlapEffect::None,
    vibrato: false,
}}}
impl BeatEffects {
    pub fn is_chord(&self) -> bool {return self.chord.is_some();}
    pub fn is_tremolo_bar(&self) -> bool {return self.tremolo_bar.is_some();}
    pub fn is_slap_effect(&self) -> bool {return self.slap_effect != SlapEffect::None;}
    pub fn has_pick_stroke(&self) -> bool {return self.pick_stroke != BeatStrokeDirection::None;}
    pub fn is_default(&self) -> bool {
        let d = BeatEffects::default();
        return self.stroke == d.stroke &&
            self.has_rasgueado == d.has_rasgueado &&
            self.pick_stroke == d.pick_stroke &&
            self.fade_in == d.fade_in &&
            self.vibrato == d.vibrato &&
            self.tremolo_bar == d.tremolo_bar &&
            self.slap_effect == d.slap_effect;
    }
}

/// Read beat effects. The first byte is effects flags:
/// - *0x01*: vibrato- *0x02*: wide vibrato
/// - *0x04*: natural harmonic
/// - *0x08*: artificial harmonic
/// - *0x10*: fade in
/// - *0x20*: tremolo bar or slap effect
/// - *0x40*: beat stroke direction
/// - *0x80*: *blank*
/// - Tremolo bar or slap effect: `byte`. If it's 0 then tremolo bar should be read (see `TremoloBar::read()`). Else it's tapping and values of the byte map to:
/// - *1*: tap
/// - *2*: slap
/// - *3*: pop
/// - Beat stroke direction. See `BeatStroke::read()`
pub fn read_beat_effects(data: &Vec<u8>, seek: &mut usize, note_effect: &mut NoteEffect) -> BeatEffects {
    let mut be = BeatEffects::default();
    let flags = read_byte(data, seek);
    note_effect.vibrato = (flags & 0x01) == 0x01 || note_effect.vibrato;
    be.vibrato = (flags & 0x02) == 0x02 || be.vibrato;
    be.fade_in = (flags & 0x10) == 0x10;
    if (flags & 0x20) == 0x20 {
        be.slap_effect = match read_byte(data, seek) {
            0 => SlapEffect::None,
            1 => SlapEffect::Tapping,
            2 => SlapEffect::Slapping,
            3 => SlapEffect::Popping,
            _ => panic!("Cannot read slap effect for the beat effects"),
        };
        if be.slap_effect == SlapEffect::None {be.tremolo_bar = Some(read_tremolo_bar(data, seek));} else {read_int(data, seek);}
        if (flags & 0x40) == 0x40 {be.stroke = read_beat_stroke(data, seek);}
        //In GP3 harmonics apply to the whole beat, not the individual notes. Here we set the noteEffect for all the notes in the beat.
        if (flags & 0x04) == 0x04 {note_effect.harmonic = Some(HarmonicEffect::default());}
        if (flags & 0x08) == 0x08 {note_effect.harmonic = Some(HarmonicEffect {kind: HarmonicType::Artificial, ..Default::default()});}
    }
    return be;
}
/// Read beat stroke. Beat stroke consists of two :ref:`Bytes <byte>` which correspond to stroke up
/// and stroke down speed. See `BeatStrokeDirection` for value mapping.
pub fn read_beat_stroke(data: &Vec<u8>, seek: &mut usize) -> BeatStroke {
    let mut bs = BeatStroke::default();
    let down = read_signed_byte(data, seek);
    let up = read_signed_byte(data, seek);
    if up > 0 {
        bs.direction = BeatStrokeDirection::Up;
        bs.value = stroke_value(up).to_u16().unwrap();
    }
    if down > 0 {
        bs.direction = BeatStrokeDirection::Down;
        bs.value = stroke_value(down).to_u16().unwrap();
    }
    return bs;
}
pub fn stroke_value(value: i8) -> u8 {
    match value {
        1 => DURATION_HUNDRED_TWENTY_EIGHTH,
        2 => DURATION_SIXTY_FOURTH,
        3 => DURATION_THIRTY_SECOND,
        4 => DURATION_SIXTEENTH,
        5 => DURATION_EIGHTH,
        6 => DURATION_QUARTER,
        _ => DURATION_SIXTY_FOURTH,
    }
}
/// Read tremolo bar beat effect. The only type of tremolo bar effect Guitar Pro 3 supports is `dip <BendType::Dip>`. The value of the
/// effect is encoded in :ref:`Int` and shows how deep tremolo bar is pressed.
pub fn read_tremolo_bar(data: &Vec<u8>, seek: &mut usize) -> BendEffect {
    let mut be = BendEffect::default();
    be.kind = BendType::Dip;
    be.value = read_int(data, seek).to_i16().unwrap();
    be.points.push(BendPoint{ position: 0, value: 0, ..Default::default() });
    be.points.push(BendPoint{ position: BEND_EFFECT_MAX_POSITION / 2,
                                    value: (-f32::from(be.value) / GP_BEND_SEMITONE).round().to_i8().unwrap(),
                                ..Default::default() });
    be.points.push(BendPoint{ position: BEND_EFFECT_MAX_POSITION, value: 0, ..Default::default() });
    return be;
}

/// A voice contains multiple beats
#[derive(Clone)]
pub struct Voice {
    //pub measure: Measure, //circular depth?
    pub measure_index: i16,
    pub beats: Vec<Beat>,
    pub directions: VoiceDirection,
}
impl Default for Voice {fn default() -> Self { Voice { measure_index: 0, /*measure: Measure::default(),*/ beats: Vec::new(), directions: VoiceDirection::None }}}

