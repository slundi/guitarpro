use std::cmp::{min,max};
use fraction::ToPrimitive;

use crate::{effects::*, enums::*, io::*, gp::*, beat::*, key_signature::*};

#[derive(Debug,Clone, PartialEq)]
pub struct Note {
    pub value: i16,
    pub velocity: i16,
    pub string: i8,
    pub effect: NoteEffect,
    pub duration_percent: f32,
    pub swap_accidentals: bool,
    pub kind: NoteType,
}
impl Default for Note {fn default() -> Self {Note {
    value: 0,
    velocity: DEFAULT_VELOCITY,
    string: 0,
    effect: NoteEffect::default(),
    duration_percent:1.0,
    swap_accidentals: false,
    kind: NoteType::Rest,
}}}
impl Note {
    pub(crate) fn real_value(&self, strings:Vec<(i8,i8)>) -> i8 {
        if self.string > 0 {return self.value.to_i8().unwrap() + strings[self.string.to_usize().unwrap() -1].1;}
        panic!("Cannot get real value for the note.");
    }
}

/// Contains all effects which can be applied to one note.
#[derive(Debug,Clone, PartialEq)]
pub struct NoteEffect {
    pub accentuated_note: bool,
    pub bend: Option<BendEffect>,
    pub ghost_note: bool,
    pub grace: Option<GraceEffect>,
    pub hammer: bool,
    pub harmonic: Option<HarmonicEffect>,
    pub heavy_accentuated_note: bool,
    pub left_hand_finger: Fingering,
    pub let_ring: bool,
    pub palm_mute: bool,
    pub right_hand_finger: Fingering,
    pub slides: Vec<SlideType>,
    pub staccato: bool,
    pub tremolo_picking: Option<TremoloPickingEffect>,
    pub trill: Option<TrillEffect>,
    pub vibrato: bool,
}
impl Default for NoteEffect {
    fn default() -> Self {NoteEffect {
        accentuated_note: false,
        bend: None,
        ghost_note: false,
        grace: None,
        hammer: false,
        harmonic: None,
        heavy_accentuated_note: false,
        left_hand_finger: Fingering::Open,
        let_ring: false,
        palm_mute: false,
        right_hand_finger: Fingering::Open,
        slides: Vec::new(),
        staccato: false,
        tremolo_picking: None,
        trill: None,
        vibrato: false,
    }}
}
impl NoteEffect {
    pub(crate) fn is_bend(&self) -> bool {self.bend.is_some()}
    pub(crate) fn is_harmonic(&self) -> bool {self.harmonic.is_some()}
    pub(crate) fn is_grace(&self) -> bool {self.grace.is_some()}
    pub(crate) fn is_trill(&self) -> bool {self.trill.is_some()}
    pub(crate) fn is_tremollo_picking(&self) -> bool {self.tremolo_picking.is_some()}
    pub(crate) fn is_default(&self) -> bool {
        let d = NoteEffect::default();
        self.left_hand_finger == d.left_hand_finger &&
        self.right_hand_finger == d.right_hand_finger &&
        self.bend == d.bend &&
        self.harmonic == d.harmonic &&
        self.grace == d.grace &&
        self.trill == d.trill &&
        self.tremolo_picking == d.tremolo_picking &&
        self.vibrato == d.vibrato &&
        self.slides == d.slides &&
        self.hammer == d.hammer &&
        self.palm_mute == d.palm_mute &&
        self.staccato == d.staccato &&
        self.let_ring == d.let_ring
    }
    pub(crate) fn is_fingering(&self) -> bool {self.left_hand_finger != Fingering::Open || self.right_hand_finger != Fingering::Open}
}

impl Song {
    /// Read notes. First byte lists played strings:
    /// - *0x01*: 7th string
    /// - *0x02*: 6th string
    /// - *0x04*: 5th string
    /// - *0x08*: 4th string
    /// - *0x10*: 3th string
    /// - *0x20*: 2th string
    /// - *0x40*: 1th string
    /// - *0x80*: *blank*
    pub(crate) fn read_notes(&mut self, data: &[u8], seek: &mut usize, track_index: usize, beat: &mut Beat, duration: &Duration, note_effect: NoteEffect) {
        let flags = read_byte(data, seek);
        println!("read_notes(), flags: {}", flags);
        for i in 0..self.tracks[track_index].strings.len() {
            if (flags & 1 << (7 - self.tracks[track_index].strings[i].0)) > 0 {
                let mut note = Note{effect: note_effect.clone(), ..Default::default()};
                if self.version.number < (5,0,0) {self.read_note(data, seek, &mut note, self.tracks[track_index].strings[i], track_index);}
                else {self.read_note_v5(data, seek, &mut note, self.tracks[track_index].strings[i], track_index);}
                beat.notes.push(note);
            }
            beat.duration = duration.clone();
        }
    }

    /// Read note. The first byte is note flags:
    /// - *0x01*: time-independent duration
    /// - *0x02*: heavy accentuated note
    /// - *0x04*: ghost note
    /// - *0x08*: presence of note effects
    /// - *0x10*: dynamics
    /// - *0x20*: fret
    /// - *0x40*: accentuated note
    /// - *0x80*: right hand or left hand fingering
    /// 
    /// Flags are followed by:
    /// - Note type: `byte`. Note is normal if values is 1, tied if value is 2, dead if value is 3.
    /// - Time-independent duration: 2 `SignedBytes <signed-byte>`. Correspond to duration and tuplet. See `read_duration()` for reference.
    /// - Note dynamics: `signed-byte`. See `unpack_velocity()`.
    /// - Fret number: `signed-byte`. If flag at *0x20* is set then read fret number.
    /// - Fingering: 2 `SignedBytes <signed-byte>`. See `Fingering`.
    /// - Note effects. See `read_note_effects()`.
    fn read_note(&mut self, data: &[u8], seek: &mut usize, note: &mut Note, guitar_string: (i8,i8), track_index: usize) {
        let flags = read_byte(data, seek);
        note.string = guitar_string.0;
        note.effect.ghost_note = (flags & 0x04) == 0x04;
        //println!("read_note(), flags: {} \t string: {} \t ghost note: {}", flags, guitar_string.0, note.effect.ghost_note);
        if (flags & 0x20) == 0x20 {note.kind = get_note_type(read_byte(data, seek)); }
        if (flags & 0x01) == 0x01 {
            let _duration = read_signed_byte(data, seek);
            let _tuplet = read_signed_byte(data, seek);
            //println!("read_note(), duration: {} \t tuplet: {}",duration, tuplet);
            //note.duration = read_signed_byte(data, seek);
            //note.tuplet = read_signed_byte(data, seek);
        }
        if (flags & 0x10) == 0x10 {
            let v = read_signed_byte(data, seek);
            //println!("read_note(), v: {}", v);
            note.velocity = crate::effects::unpack_velocity(v.to_i16().unwrap());
            //println!("read_note(), velocity: {}", note.velocity);
        }
        if (flags & 0x20) == 0x20 {
            let fret = read_signed_byte(data, seek);
            let value = if note.kind == NoteType::Tie { self.get_tied_note_value(guitar_string.0, track_index)} else {fret.to_i16().unwrap()};
            note.value = max(0, min(99, value));
            //println!("read_note(), value: {}", note.value);
        }
        if (flags & 0x80) == 0x80 {
            note.effect.left_hand_finger = get_fingering(read_signed_byte(data, seek));
            note.effect.right_hand_finger= get_fingering(read_signed_byte(data, seek));
        }
        if (flags & 0x08) == 0x08 {
            if      self.version.number == (3,0,0) {self.read_note_effects_v3(data, seek, note);}
            else if self.version.number.0 == 4 {self.read_note_effects_v4(data, seek, note);}
            if note.effect.is_harmonic() && note.effect.harmonic.is_some() {
                let mut h = note.effect.harmonic.take().unwrap();
                if h.kind == HarmonicType::Tapped {h.fret = Some(note.value.to_i8().unwrap() + 12);}
                note.effect.harmonic = Some(h);
            }
        }
    }
    /// Read note. The first byte is note flags:
    /// - *0x01*: duration percent
    /// - *0x02*: heavy accentuated note
    /// - *0x04*: ghost note
    /// - *0x08*: presence of note effects
    /// - *0x10*: dynamics
    /// - *0x20*: fret
    /// - *0x40*: accentuated note
    /// - *0x80*: right hand or left hand fingering
    /// 
    /// Flags are followed by:
    /// - Note type: `byte`. Note is normal if values is 1, tied if value is 2, dead if value is 3.
    /// - Note dynamics: `signed-byte`. See `unpackVelocity`.
    /// - Fret number: `signed-byte`. If flag at *0x20* is set then read fret number.
    /// - Fingering: 2 `SignedBytes <signed-byte>`. See :class:`Fingering`.
    /// - Duration percent: `double`.
    /// - Second set of flags: `byte`.
    /// - *0x02*: swap accidentals.
    /// - Note effects. See `read_note_effects()`.
    fn read_note_v5(&mut self, data: &[u8], seek: &mut usize, note: &mut Note, guitar_string: (i8,i8), track_index: usize) {
        let flags = read_byte(data, seek);
        note.string = guitar_string.0;
        note.effect.heavy_accentuated_note = (flags &0x02) == 0x02;
        note.effect.ghost_note = (flags &0x04) == 0x04;
        note.effect.accentuated_note = (flags &0x40) == 0x40;
        if (flags &0x20) == 0x20 {note.kind = get_note_type(read_byte(data, seek));}
        if (flags &0x10) == 0x10 {
            let v = read_signed_byte(data, seek);
            //println!("read_note(), v: {}", v);
            note.velocity = crate::effects::unpack_velocity(v.to_i16().unwrap());
            //println!("read_note(), velocity: {}", note.velocity);
        }
        if (flags &0x20) == 0x20 {
            let fret = read_signed_byte(data, seek);
            let value = if note.kind == NoteType::Tie { self.get_tied_note_value(guitar_string.0, track_index)} else {fret.to_i16().unwrap()};
            note.value = max(0, min(99, value));
            //println!("read_note(), value: {}", note.value);
        }
        if (flags &0x80) == 0x80 {
            note.effect.left_hand_finger = get_fingering(read_signed_byte(data, seek));
            note.effect.right_hand_finger= get_fingering(read_signed_byte(data, seek));
        }
        if (flags & 0x01) == 0x01 {note.duration_percent = read_double(data, seek).to_f32().unwrap();}
        note.swap_accidentals = (read_byte(data, seek) & 0x02) == 0x02;
        if (flags & 0x08) == 0x08 {self.read_note_effects_v4(data, seek, note);}
    }

    /// Read note effects. First byte is note effects flags:
    /// - *0x01*: bend presence
    /// - *0x02*: hammer-on/pull-off
    /// - *0x04*: slide
    /// - *0x08*: let-ring
    /// - *0x10*: grace note presence
    /// 
    /// Flags are followed by:
    /// - Bend. See `readBend`.
    /// - Grace note. See `readGrace`.
    fn read_note_effects_v3(&self, data: &[u8], seek: &mut usize, note: &mut Note) {
        let flags = read_byte(data, seek);
        //println!("read_effect(), flags: {}", flags);
        note.effect.hammer = (flags & 0x02) == 0x02;
        note.effect.let_ring = (flags & 0x08) == 0x08;
        if (flags & 0x01) == 0x01 {note.effect.bend = self.read_bend_effect(data, seek);}
        if (flags & 0x10) == 0x10 {note.effect.grace = Some(self.read_grace_effect(data, seek));}
        if (flags & 0x04) == 0x04 {note.effect.slides.push(SlideType::ShiftSlideTo);}
        //println!("read_note_effects(): {:?}", note);
    }
    /// Read note effects. The effects presence for the current note is set by the 2 bytes of flags. First set of flags:
    /// - *0x01*: bend
    /// - *0x02*: hammer-on/pull-off
    /// - *0x04*: *blank*
    /// - *0x08*: let-ring
    /// - *0x10*: grace note
    /// - *0x20*: *blank*
    /// - *0x40*: *blank*
    /// - *0x80*: *blank*
    /// 
    /// Second set of flags:
    /// - *0x01*: staccato
    /// - *0x02*: palm mute
    /// - *0x04*: tremolo picking
    /// - *0x08*: slide
    /// - *0x10*: harmonic
    /// - *0x20*: trill
    /// - *0x40*: vibrato
    /// - *0x80*: *blank*
    /// 
    /// Flags are followed by:
    /// - Bend. See `read_bend()`.
    /// - Grace note. See `read_grace()`.
    /// - Tremolo picking. See `read_tremolo_picking()`.
    /// - Slide. See `read_slides()`.
    /// - Harmonic. See `read_harmonic()`.
    /// - Trill. See `read_trill()`.
    fn read_note_effects_v4(&mut self, data: &[u8], seek: &mut usize, note: &mut Note) {
        let flags1 = read_signed_byte(data, seek);
        let flags2 = read_signed_byte(data, seek);
        note.effect.hammer = (flags1 & 0x02) == 0x02;
        note.effect.let_ring = (flags1 & 0x08) == 0x08;
        note.effect.staccato = (flags2 & 0x01) == 0x01;
        note.effect.palm_mute = (flags2 & 0x02) == 0x02;
        note.effect.vibrato = (flags2 & 0x40) == 0x40 || note.effect.vibrato;
        if (flags1 & 0x01) == 0x01 {note.effect.bend = self.read_bend_effect(data, seek);}
        if (flags1 & 0x10) == 0x10 {note.effect.grace = Some(self.read_grace_effect(data, seek));}
        if (flags2 & 0x04) == 0x04 {note.effect.tremolo_picking = Some(self.read_tremolo_picking(data, seek));}
        if (flags2 & 0x08) == 0x08 {note.effect.slides.push(get_slide_type(read_signed_byte(data, seek)));}
        if (flags2 & 0x10) == 0x10 {note.effect.harmonic = Some(self.read_harmonic(data, seek, note));}
        if (flags2 & 0x20) == 0x20 {note.effect.trill = Some(self.read_trill(data, seek));}
    }

    /// Get note value of tied note
    fn get_tied_note_value(&self, string_index: i8, track_index: usize) -> i16 {
        //println!("get_tied_note_value()");
        for m in (0usize..self.tracks[track_index].measures.len()).rev() {
            for v in (0usize..self.tracks[track_index].measures[m].voices.len()).rev() {
                for b in 0..self.tracks[track_index].measures[m].voices[v].beats.len() {
                    if self.tracks[track_index].measures[m].voices[v].beats[b].status != BeatStatus::Empty {
                        for n in 0..self.tracks[track_index].measures[m].voices[v].beats[b].notes.len() {
                            if self.tracks[track_index].measures[m].voices[v].beats[b].notes[n].string == string_index {return self.tracks[track_index].measures[m].voices[v].beats[b].notes[n].value;}
                        }
                    }
                }
            }
        }
        -1
    }
}