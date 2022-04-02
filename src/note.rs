use crate::{effects::*, chord::*};


/// Contains all effects which can be applied to one note.
#[derive(Clone, PartialEq)]
pub struct NoteEffect {
    accentuated_note: bool,
    bend: Option<BendEffect>,
    ghost_note: bool,
    grace: Option<GraceEffect>,
    hammer: bool,
    harmonic: Option<HarmonicEffect>,
    heavy_accentuated_note: bool,
    left_hand_finger: Fingering,
    let_ring: bool,
    palm_mute: bool,
    right_hand_finger: Fingering,
    slides: Vec<SlideType>,
    staccato: bool,
    tremolo_picking: Option<TremoloPickingEffect>,
    trill: Option<TrillEffect>,
    vibrato: bool,
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

/// An enumeration of all supported slide types.
#[derive(Clone,PartialEq)]
pub enum SlideType {
    IntoFromAbove, //-2
    IntoFromBelow, //-1
    None, //0
    ShiftSlideTo,
    LegatoSlideTo,
    OutDownwards,
    OutUpWards
}

/// An enumeration of all supported slide types.
#[derive(Clone)]
pub enum NoteType {
    Rest, //0
    Normal, Tie, Dead,
}

