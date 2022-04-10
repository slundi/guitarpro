use fraction::ToPrimitive;

use crate::{mix_table::*, effects::*, chord::*, key_signature::*, note::*, io::*, gp::*, enums::*};

/// Parameters of beat display
#[derive(Debug,Clone,PartialEq)]
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
#[derive(Debug,Clone,PartialEq)]
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


/// A voice contains multiple beats
#[derive(Debug,Clone)]
pub struct Voice {
    //pub measure: Measure, //circular depth?
    pub measure_index: i16,
    pub beats: Vec<Beat>,
    pub directions: VoiceDirection,
}
impl Default for Voice {fn default() -> Self { Voice { measure_index: 0, /*measure: Measure::default(),*/ beats: Vec::new(), directions: VoiceDirection::None }}}

/// This class contains all beat effects
#[derive(Debug,Clone,PartialEq)]
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

/// A beat contains multiple notes
#[derive(Debug,Clone,PartialEq)]
pub struct Beat {
    pub notes: Vec<Note>,
    pub duration: Duration,
    pub text: String,
    pub start: Option<i64>,
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
    status: BeatStatus::Normal,
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

impl Song {
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
    pub fn read_beat(&mut self, data: &Vec<u8>, seek: &mut usize, voice: &mut Voice, start: i64, track_index: usize) -> i64 {
        let flags = read_byte(data, seek);
        //println!("read_beat(), flags: {}", flags);
        //get a beat
        let mut b = 0;
        let mut new_beat = true;
        for i in (0usize..voice.beats.len()).rev() {if voice.beats[i].start == Some(start) {
            b = i;
            new_beat = false;
            break;
        }}
        if new_beat {
            voice.beats.push(Beat{start: Some(start), ..Default::default() });
            b = voice.beats.len() - 1;
        }
        
        if (flags & 0x40) == 0x40 { voice.beats[b].status = get_beat_status(read_byte(data, seek));} //else { voice.beats[b].status = BeatStatus::Normal;}
        let duration = read_duration(data, seek, flags);
        let mut note_effect = NoteEffect::default();
        if (flags & 0x02) == 0x02 {voice.beats[b].effect.chord = Some(self.read_chord(data, seek, self.tracks[track_index].strings.len().to_u8().unwrap()));}
        if (flags & 0x04) == 0x04 {voice.beats[b].text = read_int_size_string(data, seek);}
        if (flags & 0x08) == 0x08 {
            let chord = voice.beats[b].effect.chord.clone();
            if      self.version.number == AppVersion::Version_3_00 {voice.beats[b].effect = self.read_beat_effects_v3(data, seek, &mut note_effect); }
            else if self.version.number == AppVersion::Version_4_0x {voice.beats[b].effect = self.read_beat_effects_v4(data, seek, &mut note_effect);}
            voice.beats[b].effect.chord = chord;
        }
        if (flags & 0x10) == 0x10 {
            let mtc = self.read_mix_table_change(data, seek);
            voice.beats[b].effect.mix_table_change = Some(mtc);
        }
        self.read_notes(data, seek, track_index, &mut voice.beats[b], &duration, note_effect);
        if voice.beats[b].status == BeatStatus::Empty {return 0;} else {return duration.time().to_i64().unwrap();}
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
    fn read_beat_effects_v3(&self, data: &Vec<u8>, seek: &mut usize, note_effect: &mut NoteEffect) -> BeatEffects {
        //println!("read_beat_effects()");
        let mut be = BeatEffects::default();
        let flags = read_byte(data, seek);
        note_effect.vibrato = (flags & 0x01) == 0x01 || note_effect.vibrato;
        be.vibrato = (flags & 0x02) == 0x02 || be.vibrato;
        be.fade_in = (flags & 0x10) == 0x10;
        if (flags & 0x20) == 0x20 {
            be.slap_effect = get_slap_effect(read_byte(data, seek));
            if be.slap_effect == SlapEffect::None {be.tremolo_bar = Some(self.read_tremolo_bar(data, seek));} else {read_int(data, seek);}
        }
        if (flags & 0x40) == 0x40 {be.stroke = self.read_beat_stroke(data, seek);}
        //In GP3 harmonics apply to the whole beat, not the individual notes. Here we set the noteEffect for all the notes in the beat.
        if (flags & 0x04) == 0x04 {note_effect.harmonic = Some(HarmonicEffect::default());}
        if (flags & 0x08) == 0x08 {note_effect.harmonic = Some(HarmonicEffect {kind: HarmonicType::Artificial, ..Default::default()});}
        return be;
    }
    ///Read beat effects. Beat effects are read using two byte flags. The first byte of flags is:
    /// - *0x01*: *blank*
    /// - *0x02*: wide vibrato
    /// - *0x04*: *blank*
    /// - *0x08*: *blank*
    /// - *0x10*: fade in
    /// - *0x20*: slap effect
    /// - *0x40*: beat stroke
    /// - *0x80*: *blank*
    /// 
    /// The second byte of flags is:
    /// - *0x01*: rasgueado
    /// - *0x02*: pick stroke
    /// - *0x04*: tremolo bar
    /// - *0x08*: *blank*
    /// - *0x10*: *blank*
    /// - *0x20*: *blank*
    /// - *0x40*: *blank*
    /// - *0x80*: *blank*
    /// 
    /// Flags are followed by:
    /// - Slap effect: `signed-byte`. For value mapping see `SlapEffect`.
    /// - Tremolo bar. See `readTremoloBar`.
    /// - Beat stroke. See `readBeatStroke`.
    /// - Pick stroke: `signed-byte`. For value mapping see `BeatStrokeDirection`.
    fn read_beat_effects_v4(&self, data: &Vec<u8>, seek: &mut usize, note_effect: &mut NoteEffect) -> BeatEffects {
        let mut be = BeatEffects::default();
        let flags1 = read_signed_byte(data, seek);
        let flags2 = read_signed_byte(data, seek);
        note_effect.vibrato = (flags1 & 0x01) == 0x01 || note_effect.vibrato;
        be.vibrato = (flags1 & 0x02) == 0x02 || be.vibrato;
        be.fade_in = (flags1 & 0x10) == 0x10;
        if (flags1 & 0x20) == 0x20 {be.slap_effect = get_slap_effect(read_signed_byte(data, seek).to_u8().unwrap());}
        if (flags2 & 0x04) == 0x04 {be.tremolo_bar = self.read_bend_effect(data, seek);}
        if (flags1 & 0x40) == 0x40 {be.stroke = self.read_beat_stroke(data, seek);}
        be.has_rasgueado = (flags2 &0x01) == 0x01;
        if (flags2 & 0x02) == 0x02 {be.pick_stroke = get_beat_stroke_direction(read_signed_byte(data, seek));}
        return be;
    }
    /// Read beat stroke. Beat stroke consists of two `Bytes <byte>` which correspond to stroke up
    /// and stroke down speed. See `BeatStrokeDirection` for value mapping.
    fn read_beat_stroke(&self, data: &Vec<u8>, seek: &mut usize) -> BeatStroke {
        //println!("read_beat_stroke()");
        let mut bs = BeatStroke::default();
        let down = read_signed_byte(data, seek);
        let up = read_signed_byte(data, seek);
        if up > 0 {
            bs.direction = BeatStrokeDirection::Up;
            bs.value = self.stroke_value(up).to_u16().unwrap();
        }
        if down > 0 {
            bs.direction = BeatStrokeDirection::Down;
            bs.value = self.stroke_value(down).to_u16().unwrap();
        }
        return bs;
    }
    fn stroke_value(&self, value: i8) -> u8 {
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
    /// effect is encoded in `Int` and shows how deep tremolo bar is pressed.
    fn read_tremolo_bar(&self, data: &Vec<u8>, seek: &mut usize) -> BendEffect {
        //println!("read_tremolo_bar()");
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
}