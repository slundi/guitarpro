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
    pub(crate) fn swap_direction(&mut self) {
        if self.direction == BeatStrokeDirection::Up {self.direction = BeatStrokeDirection::Down}
        else if self.direction == BeatStrokeDirection::Down {self.direction = BeatStrokeDirection::Up}
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
    pub(crate) fn is_chord(&self) -> bool {self.chord.is_some()}
    pub(crate) fn is_tremolo_bar(&self) -> bool {self.tremolo_bar.is_some()}
    pub(crate) fn is_slap_effect(&self) -> bool {self.slap_effect != SlapEffect::None}
    pub(crate) fn has_pick_stroke(&self) -> bool {self.pick_stroke != BeatStrokeDirection::None}
    pub(crate) fn is_default(&self) -> bool {
        let d = BeatEffects::default();
        self.stroke == d.stroke &&
            self.has_rasgueado == d.has_rasgueado &&
            self.pick_stroke == d.pick_stroke &&
            self.fade_in == d.fade_in &&
            self.vibrato == d.vibrato &&
            self.tremolo_bar == d.tremolo_bar &&
            self.slap_effect == d.slap_effect
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
    //pub(crate) fn start_in_measure(&self) -> u16 {self.start - self.voice.measure.start}
    pub(crate) fn has_vibrato(&self) -> bool {
        for i in 0..self.notes.len() {if self.notes[i].effect.vibrato {return true;}}
        false
    }
    pub(crate) fn has_harmonic(&self) -> bool {
        for i in 0..self.notes.len() {if self.notes[i].effect.is_harmonic() {return true;}}
        false
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
    pub(crate) fn read_beat(&mut self, data: &[u8], seek: &mut usize, voice: &mut Voice, start: i64, track_index: usize) -> i64 {
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
            if      self.version.number == (3,0,0) {voice.beats[b].effect = self.read_beat_effects_v3(data, seek, &mut note_effect); }
            else if self.version.number.0 == 4 {voice.beats[b].effect = self.read_beat_effects_v4(data, seek, &mut note_effect);}
            voice.beats[b].effect.chord = chord;
        }
        if (flags & 0x10) == 0x10 {
            let mtc = self.read_mix_table_change(data, seek);
            voice.beats[b].effect.mix_table_change = Some(mtc);
        }
        self.read_notes(data, seek, track_index, &mut voice.beats[b], &duration, note_effect);
        if voice.beats[b].status == BeatStatus::Empty {0} else {duration.time().to_i64().unwrap()}
    }
    /// Read beat. First, beat is read is in Guitar Pro 3 `guitarpro.gp3.readBeat`. Then it is followed by set of flags stored in `short`.
    /// - *0x0001*: break beams
    /// - *0x0002*: direct beams down
    /// - *0x0004*: force beams
    /// - *0x0008*: direct beams up
    /// - *0x0010*: ottava (8va)
    /// - *0x0020*: ottava bassa (8vb)
    /// - *0x0040*: quindicesima (15ma)
    /// - *0x0100*: quindicesima bassa (15mb)
    /// - *0x0200*: start tuplet bracket here
    /// - *0x0400*: end tuplet bracket here
    /// - *0x0800*: break secondary beams
    /// - *0x1000*: break secondary tuplet
    /// - *0x2000*: force tuplet bracket
    /// - Break secondary beams: `byte`. Appears if flag at *0x0800* is set. Signifies how much beams should be broken.
    pub(crate) fn read_beat_v5(&mut self, data: &[u8], seek: &mut usize, start: &mut i64, voice: &mut Voice) -> i64 {
        //TODO: let duration = self.read_beat(data, seek, voice, start, track_index);
        //get a beat
        let mut b = 0;
        let mut new_beat = true;
        for i in (0usize..voice.beats.len()).rev() {if voice.beats[i].start == Some(*start) {
            b = i;
            new_beat = false;
            break;
        }}
        if new_beat {
            voice.beats.push(Beat{start: Some(*start), ..Default::default() });
            b = voice.beats.len() - 1;
        }

        let flags2 = read_short(data, seek);
        if (flags2 & 0x0010) == 0x0010 {voice.beats[b].octave = Octave::Ottava;}
        if (flags2 & 0x0020) == 0x0020 {voice.beats[b].octave = Octave::OttavaBassa;}
        if (flags2 & 0x0040) == 0x0040 {voice.beats[b].octave = Octave::Quindicesima;}
        if (flags2 & 0x0100) == 0x0100 {voice.beats[b].octave = Octave::QuindicesimaBassa;}

        voice.beats[b].display.break_beam = (flags2 & 0x0001) == 0x0001;
        voice.beats[b].display.force_beam = (flags2 & 0x0004) == 0x0004;
        voice.beats[b].display.force_bracket = (flags2 & 0x2000) == 0x2000;
        voice.beats[b].display.break_secondary_tuplet = (flags2 & 0x1000) == 0x1000;
        if (flags2 & 0x0002) == 0x0002 {voice.beats[b].display.beam_direction = VoiceDirection::Down;}
        if (flags2 & 0x0008) == 0x0008 {voice.beats[b].display.beam_direction = VoiceDirection::Up;}
        if (flags2 & 0x0400) == 0x0400 {voice.beats[b].display.tuple_bracket = TupletBracket::Start;}
        if (flags2 & 0x0800) == 0x0800 {voice.beats[b].display.tuple_bracket = TupletBracket::End;}

        0//duration
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
    fn read_beat_effects_v3(&self, data: &[u8], seek: &mut usize, note_effect: &mut NoteEffect) -> BeatEffects {
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
        be
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
    fn read_beat_effects_v4(&self, data: &[u8], seek: &mut usize, note_effect: &mut NoteEffect) -> BeatEffects {
        let mut be = BeatEffects::default();
        let flags1 = read_signed_byte(data, seek);
        let flags2 = read_signed_byte(data, seek);
        be.vibrato = (flags1 & 0x02) == 0x02 || be.vibrato;
        be.fade_in = (flags1 & 0x10) == 0x10;
        if (flags1 & 0x20) == 0x20 {be.slap_effect = get_slap_effect(read_signed_byte(data, seek).to_u8().unwrap());}
        if (flags2 & 0x04) == 0x04 {be.tremolo_bar = self.read_bend_effect(data, seek);}
        if (flags1 & 0x40) == 0x40 {be.stroke = self.read_beat_stroke(data, seek);}
        be.has_rasgueado = (flags2 &0x01) == 0x01;
        if (flags2 & 0x02) == 0x02 {be.pick_stroke = get_beat_stroke_direction(read_signed_byte(data, seek));}
        //println!("Beat effect: {:?}", be);
        be
    }
    /// Read beat stroke. Beat stroke consists of two `Bytes <byte>` which correspond to stroke up
    /// and stroke down speed. See `BeatStrokeDirection` for value mapping.
    fn read_beat_stroke(&self, data: &[u8], seek: &mut usize) -> BeatStroke {
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
        if self.version.number >= (5,0,0) {bs.swap_direction();}
        bs
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
    fn read_tremolo_bar(&self, data: &[u8], seek: &mut usize) -> BendEffect {
        //println!("read_tremolo_bar()");
        let mut be = BendEffect{kind: BendType::Dip, ..Default::default()};
        be.value = read_int(data, seek).to_i16().unwrap();
        be.points.push(BendPoint{ position: 0, value: 0, ..Default::default() });
        be.points.push(BendPoint{ position: BEND_EFFECT_MAX_POSITION / 2,
                                        value: (-f32::from(be.value) / GP_BEND_SEMITONE).round().to_i8().unwrap(),
                                    ..Default::default() });
        be.points.push(BendPoint{ position: BEND_EFFECT_MAX_POSITION, value: 0, ..Default::default() });
        be
    }
}