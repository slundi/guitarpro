use std::cmp::{min,max};
use fraction::ToPrimitive;

use crate::{effects::*, enums::*, io::*, track::*, beat::*, key_signature::*};

#[derive(Clone, PartialEq)]
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
    /*pub fn real_value(&self) {
        return self.value + self.beat.voice.measure.track.strings[self.string -1].value;
    }*/
}

/// Contains all effects which can be applied to one note.
#[derive(Clone, PartialEq)]
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
    pub fn is_bend(&self) -> bool {return self.bend.is_some();}
    pub fn is_harmonic(&self) -> bool {return self.harmonic.is_some();}
    pub fn is_grace(&self) -> bool {return self.grace.is_some();}
    pub fn is_trill(&self) -> bool {return self.trill.is_some();}
    pub fn is_tremollo_picking(&self) -> bool {return self.tremolo_picking.is_some();}
    pub fn is_default(&self) -> bool {
        let d = NoteEffect::default();
        return self.left_hand_finger == d.left_hand_finger &&
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
        self.let_ring == d.let_ring;
    }
    pub fn is_fingering(&self) -> bool {return self.left_hand_finger != Fingering::Open || self.right_hand_finger != Fingering::Open;}
}

/// Read notes. First byte lists played strings:
/// - *0x01*: 7th string
/// - *0x02*: 6th string
/// - *0x04*: 5th string
/// - *0x08*: 4th string
/// - *0x10*: 3th string
/// - *0x20*: 2th string
/// - *0x40*: 1th string
/// - *0x80*: *blank*
pub fn read_notes(data: &Vec<u8>, seek: &mut usize, track: &mut Track, beat: &mut Beat, duration: &Duration, note_effect: NoteEffect) {
    println!("read_notes()");
    let flags = read_byte(data, seek);
    for i in 0..track.strings.len() {
        if (flags & 1 << (7 - track.strings[i].0.to_u8().unwrap())) > 0 {
            let mut note = Note{effect: note_effect.clone(), ..Default::default()};
            read_note(data, seek, &mut note, track.strings[i], track);
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
fn read_note(data: &Vec<u8>, seek: &mut usize, note: &mut Note, guitar_string: (i8,i8), track: &mut Track) {
    println!("read_note()");
    let flags = read_byte(data, seek);
    note.string = guitar_string.0;
    note.effect.ghost_note = (flags & 0x04) == 0x04;
    if (flags & 0x02) == 0x02 {note.kind = match read_byte(data, seek) {
        0 => NoteType::Rest,
        1 => NoteType::Normal,
        2 => NoteType::Tie,
        3 => NoteType::Dead,
        _ => panic!("Cannot read note type"),
    }}
    if (flags & 0x01) == 0x01 {
        read_signed_byte(data, seek);read_signed_byte(data, seek);
        //note.duration = read_signed_byte(data, seek);
        //note.tuplet = read_signed_byte(data, seek);
    }
    if (flags & 0x10) == 0x10 {
        let v = read_signed_byte(data, seek);
        note.velocity = crate::effects::unpack_velocity(v.to_i16().unwrap());
    }
    if (flags & 0x20) == 0x20 {
        let fret = read_signed_byte(data, seek);
        let value = if note.kind == NoteType::Tie { get_tied_note_value(guitar_string.0, track)}
        else {fret.to_i16().unwrap()};
        note.value = max(0, min(99, value));
    }
    if (flags & 0x80) == 0x80 {
        note.effect.left_hand_finger = get_fingering(read_signed_byte(data, seek));
        note.effect.right_hand_finger= get_fingering(read_signed_byte(data, seek));
    }
    if (flags & 0x08) == 0x08 {
        read_note_effect(data, seek, note);
        if note.effect.is_harmonic() && note.effect.harmonic.is_some() {
            let mut h = note.effect.harmonic.take().unwrap();
            if h.kind == HarmonicType::Tapped {h.fret = Some(note.value.to_i8().unwrap() + 12);}
            note.effect.harmonic = Some(h);
        }
    }
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
fn read_note_effect(data: &Vec<u8>, seek: &mut usize, note: &mut Note) {
    println!("read_note_effect()");
    let flags = read_byte(data, seek);
    note.effect.hammer = (flags & 0x02) == 0x02;
    note.effect.let_ring = (flags & 0x08) == 0x08;
    if (flags & 0x01) == 0x01 {note.effect.bend = read_bend_effect(data, seek);}
    if (flags & 0x10) == 0x10 {note.effect.grace = Some(read_grace_effect(data, seek));}
    if (flags & 0x04) == 0x04 {note.effect.slides.push(SlideType::ShiftSlideTo);}
}

/// Get note value of tied note
fn get_tied_note_value(string_index: i8, track: &Track) -> i16 {
    println!("get_tied_note_value()");
    for m in (0usize..track.measures.len()).rev() {
        for v in (0usize..track.measures[m].voices.len()).rev() {
            for b in 0..track.measures[m].voices[v].beats.len() {
                if track.measures[m].voices[v].beats[b].status != BeatStatus::Empty {
                    for n in 0..track.measures[m].voices[v].beats[b].notes.len() {
                        if track.measures[m].voices[v].beats[b].notes[n].string == string_index {return track.measures[m].voices[v].beats[b].notes[n].value;}
                    }
                }
            }
        }
    }
    return -1;
}
