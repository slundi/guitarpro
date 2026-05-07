mod read;
mod write;

use crate::error::{GpError, GpResult};
use crate::model::{beat::Beat, effects::*, enums::*, key_signature::Duration, song::Song};

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub value: i16,
    pub velocity: i16,
    pub string: i8,
    pub effect: NoteEffect,
    pub duration_percent: f32,
    pub swap_accidentals: bool,
    pub kind: NoteType,
    pub duration: Option<i8>,
    pub tuplet: Option<i8>,
}
impl Default for Note {
    fn default() -> Self {
        Note {
            value: 0,
            velocity: DEFAULT_VELOCITY,
            string: 0,
            effect: NoteEffect::default(),
            duration_percent: 1.0,
            swap_accidentals: false,
            kind: NoteType::Rest,
            duration: None,
            tuplet: None,
        }
    }
}
impl Note {
    pub(crate) fn real_value(&self, strings: &[(i8, i8)]) -> GpResult<i8> {
        use crate::error::ToPrimitiveGp;
        if self.string > 0 {
            let value = self.value.to_i8_gp("note value")?;
            let string_idx = self.string.to_usize_gp("string index")? - 1;
            return Ok(value + strings[string_idx].1);
        }
        Err(GpError::InvalidValue {
            context: "note real_value (string must be > 0)",
            value: self.string as i64,
        })
    }
}

/// Contains all effects which can be applied to one note.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Ornament type from GPIF (GP6/GP7)
    pub ornament: Option<String>,
}
impl Default for NoteEffect {
    fn default() -> Self {
        NoteEffect {
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
            ornament: None,
        }
    }
}
impl NoteEffect {
    pub(crate) fn is_bend(&self) -> bool {
        self.bend.is_some()
    }
    pub(crate) fn is_harmonic(&self) -> bool {
        self.harmonic.is_some()
    }
    pub(crate) fn is_grace(&self) -> bool {
        self.grace.is_some()
    }
    pub(crate) fn is_trill(&self) -> bool {
        self.trill.is_some()
    }
    pub(crate) fn is_tremollo_picking(&self) -> bool {
        self.tremolo_picking.is_some()
    }
    pub(crate) fn is_default(&self) -> bool {
        let d = NoteEffect::default();
        self.left_hand_finger == d.left_hand_finger
            && self.right_hand_finger == d.right_hand_finger
            && self.bend == d.bend
            && self.harmonic == d.harmonic
            && self.grace == d.grace
            && self.trill == d.trill
            && self.tremolo_picking == d.tremolo_picking
            && self.vibrato == d.vibrato
            && self.slides == d.slides
            && self.hammer == d.hammer
            && self.palm_mute == d.palm_mute
            && self.staccato == d.staccato
            && self.let_ring == d.let_ring
    }
    pub(crate) fn is_fingering(&self) -> bool {
        self.left_hand_finger != Fingering::Open || self.right_hand_finger != Fingering::Open
    }
}

pub trait SongNoteOps {
    fn read_notes(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        track_index: usize,
        beat: &mut Beat,
        duration: &Duration,
        note_effect: NoteEffect,
    ) -> GpResult<()>;
    fn read_note(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        note: &mut Note,
        guitar_string: (i8, i8),
        track_index: usize,
    ) -> GpResult<()>;
    fn read_note_v5(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        note: &mut Note,
        guitar_string: (i8, i8),
        track_index: usize,
    ) -> GpResult<()>;
    fn read_note_effects_v3(&self, data: &[u8], seek: &mut usize, note: &mut Note) -> GpResult<()>;
    fn read_note_effects_v4(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        note: &mut Note,
    ) -> GpResult<()>;
    fn get_tied_note_value(&self, string_index: i8, track_index: usize) -> i16;
    fn write_notes(
        &self,
        data: &mut Vec<u8>,
        beat: &Beat,
        strings: &[(i8, i8)],
        version: &(u8, u8, u8),
    ) -> GpResult<()>;
    fn write_note_v3(&self, data: &mut Vec<u8>, note: &Note) -> GpResult<()>;
    fn write_note_v4(
        &self,
        data: &mut Vec<u8>,
        note: &Note,
        strings: &[(i8, i8)],
        version: &(u8, u8, u8),
    ) -> GpResult<()>;
    fn write_note_v5(
        &self,
        data: &mut Vec<u8>,
        note: &Note,
        strings: &[(i8, i8)],
        version: &(u8, u8, u8),
    ) -> GpResult<()>;
    fn pack_note_flags(&self, note: &Note, version: &(u8, u8, u8)) -> u8;
    fn write_note_effects_v3(&self, data: &mut Vec<u8>, note: &Note) -> GpResult<()>;
    fn write_note_effects(
        &self,
        data: &mut Vec<u8>,
        note: &Note,
        strings: &[(i8, i8)],
        version: &(u8, u8, u8),
    ) -> GpResult<()>;
}

impl SongNoteOps for Song {
    fn read_notes(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        track_index: usize,
        beat: &mut Beat,
        duration: &Duration,
        note_effect: NoteEffect,
    ) -> GpResult<()> {
        read::read_notes(self, data, seek, track_index, beat, duration, note_effect)
    }
    fn read_note(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        note: &mut Note,
        guitar_string: (i8, i8),
        track_index: usize,
    ) -> GpResult<()> {
        read::read_note(self, data, seek, note, guitar_string, track_index)
    }
    fn read_note_v5(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        note: &mut Note,
        guitar_string: (i8, i8),
        track_index: usize,
    ) -> GpResult<()> {
        read::read_note_v5(self, data, seek, note, guitar_string, track_index)
    }
    fn read_note_effects_v3(&self, data: &[u8], seek: &mut usize, note: &mut Note) -> GpResult<()> {
        read::read_note_effects_v3(self, data, seek, note)
    }
    fn read_note_effects_v4(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        note: &mut Note,
    ) -> GpResult<()> {
        read::read_note_effects_v4(self, data, seek, note)
    }
    fn get_tied_note_value(&self, string_index: i8, track_index: usize) -> i16 {
        read::get_tied_note_value(self, string_index, track_index)
    }
    fn write_notes(
        &self,
        data: &mut Vec<u8>,
        beat: &Beat,
        strings: &[(i8, i8)],
        version: &(u8, u8, u8),
    ) -> GpResult<()> {
        write::write_notes(self, data, beat, strings, version)
    }
    fn write_note_v3(&self, data: &mut Vec<u8>, note: &Note) -> GpResult<()> {
        write::write_note_v3(self, data, note)
    }
    fn write_note_v4(
        &self,
        data: &mut Vec<u8>,
        note: &Note,
        strings: &[(i8, i8)],
        version: &(u8, u8, u8),
    ) -> GpResult<()> {
        write::write_note_v4(self, data, note, strings, version)
    }
    fn write_note_v5(
        &self,
        data: &mut Vec<u8>,
        note: &Note,
        strings: &[(i8, i8)],
        version: &(u8, u8, u8),
    ) -> GpResult<()> {
        write::write_note_v5(self, data, note, strings, version)
    }
    fn pack_note_flags(&self, note: &Note, version: &(u8, u8, u8)) -> u8 {
        write::pack_note_flags(self, note, version)
    }
    fn write_note_effects_v3(&self, data: &mut Vec<u8>, note: &Note) -> GpResult<()> {
        write::write_note_effects_v3(self, data, note)
    }
    fn write_note_effects(
        &self,
        data: &mut Vec<u8>,
        note: &Note,
        strings: &[(i8, i8)],
        version: &(u8, u8, u8),
    ) -> GpResult<()> {
        write::write_note_effects(self, data, note, strings, version)
    }
}
