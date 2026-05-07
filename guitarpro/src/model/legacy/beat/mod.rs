mod read;
mod write;

use crate::error::GpResult;
use crate::model::legacy::{
    chord::Chord, effects::*, enums::*, key_signature::Duration, mix_table::MixTableChange,
    note::*, song::Song,
};

/// Parameters of beat display
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeatDisplay {
    pub(crate) break_beam: bool,
    pub(crate) force_beam: bool,
    pub(crate) beam_direction: VoiceDirection,
    pub(crate) tuplet_bracket: TupletBracket,
    pub(crate) break_secondary: u8,
    pub(crate) break_secondary_tuplet: bool,
    pub(crate) force_bracket: bool,
}
impl Default for BeatDisplay {
    fn default() -> Self {
        BeatDisplay {
            break_beam: false,
            force_beam: false,
            beam_direction: VoiceDirection::None,
            tuplet_bracket: TupletBracket::None,
            break_secondary: 0,
            break_secondary_tuplet: false,
            force_bracket: false,
        }
    }
}

/// A stroke effect for beats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeatStroke {
    pub direction: BeatStrokeDirection,
    pub value: u16,
    pub swap: bool,
}
impl Default for BeatStroke {
    fn default() -> Self {
        BeatStroke {
            direction: BeatStrokeDirection::None,
            value: 0,
            swap: false,
        }
    }
}
impl BeatStroke {
    pub(crate) fn swap_direction(&mut self) {
        if self.direction == BeatStrokeDirection::Up {
            self.direction = BeatStrokeDirection::Down
        } else if self.direction == BeatStrokeDirection::Down {
            self.direction = BeatStrokeDirection::Up
        }
    }
}

/// A voice contains multiple beats
#[derive(Debug, Clone)]
pub struct Voice {
    pub measure_index: i16,
    pub beats: Vec<Beat>,
    pub directions: VoiceDirection,
}
impl Default for Voice {
    fn default() -> Self {
        Voice {
            measure_index: 0,
            beats: Vec::new(),
            directions: VoiceDirection::None,
        }
    }
}

/// This class contains all beat effects
#[derive(Debug, Clone, PartialEq, Eq)]
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
impl Default for BeatEffects {
    fn default() -> Self {
        BeatEffects {
            stroke: BeatStroke::default(),
            has_rasgueado: false,
            pick_stroke: BeatStrokeDirection::None,
            chord: None,
            fade_in: false,
            tremolo_bar: None,
            mix_table_change: None,
            slap_effect: SlapEffect::None,
            vibrato: false,
        }
    }
}
impl BeatEffects {
    pub(crate) fn is_chord(&self) -> bool {
        self.chord.is_some()
    }
    pub(crate) fn is_tremolo_bar(&self) -> bool {
        self.tremolo_bar.is_some()
    }
    pub(crate) fn is_slap_effect(&self) -> bool {
        self.slap_effect != SlapEffect::None
    }
    pub(crate) fn has_pick_stroke(&self) -> bool {
        self.pick_stroke != BeatStrokeDirection::None
    }
    pub(crate) fn is_default(&self) -> bool {
        let d = BeatEffects::default();
        self.stroke == d.stroke
            && self.has_rasgueado == d.has_rasgueado
            && self.pick_stroke == d.pick_stroke
            && self.fade_in == d.fade_in
            && self.vibrato == d.vibrato
            && self.tremolo_bar == d.tremolo_bar
            && self.slap_effect == d.slap_effect
    }
}

/// A beat contains multiple notes
#[derive(Debug, Clone, PartialEq)]
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
impl Default for Beat {
    fn default() -> Self {
        Beat {
            notes: Vec::with_capacity(12),
            duration: Duration::default(),
            text: String::new(),
            start: None,
            effect: BeatEffects::default(),
            octave: Octave::None,
            display: BeatDisplay::default(),
            status: BeatStatus::Normal,
        }
    }
}
impl Beat {
    pub(crate) fn has_vibrato(&self) -> bool {
        for i in 0..self.notes.len() {
            if self.notes[i].effect.vibrato {
                return true;
            }
        }
        false
    }
    pub(crate) fn has_harmonic(&self) -> bool {
        for i in 0..self.notes.len() {
            if self.notes[i].effect.is_harmonic() {
                return true;
            }
        }
        false
    }
}

pub trait SongBeatOps {
    fn read_beat(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        voice: &mut Voice,
        start: i64,
        track_index: usize,
    ) -> GpResult<i64>;
    fn read_beat_v5(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        voice: &mut Voice,
        start: &mut i64,
        track_index: usize,
    ) -> GpResult<i64>;
    fn read_beat_effects_v3(
        &self,
        data: &[u8],
        seek: &mut usize,
        note_effect: &mut NoteEffect,
    ) -> GpResult<BeatEffects>;
    fn read_beat_effects_v4(&self, data: &[u8], seek: &mut usize) -> GpResult<BeatEffects>;
    fn read_beat_stroke(&self, data: &[u8], seek: &mut usize) -> GpResult<BeatStroke>;
    fn stroke_value(&self, value: i8) -> u8;
    fn read_tremolo_bar(&self, data: &[u8], seek: &mut usize) -> GpResult<BendEffect>;
    fn write_beat_v3(&self, data: &mut Vec<u8>, beat: &Beat) -> GpResult<()>;
    fn write_beat(
        &self,
        data: &mut Vec<u8>,
        beat: &Beat,
        strings: &[(i8, i8)],
        version: &(u8, u8, u8),
    ) -> GpResult<()>;
    fn write_beat_effect_v3(&self, data: &mut Vec<u8>, beat: &Beat) -> GpResult<()>;
    fn write_beat_effect_v4(
        &self,
        data: &mut Vec<u8>,
        beat: &Beat,
        version: &(u8, u8, u8),
    ) -> GpResult<()>;
    fn write_tremolo_bar(&self, data: &mut Vec<u8>, bar: &Option<BendEffect>) -> GpResult<()>;
    fn write_beat_stroke(
        &self,
        data: &mut Vec<u8>,
        stroke: &BeatStroke,
        version: &(u8, u8, u8),
    ) -> GpResult<()>;
    fn from_stroke_value(value: u8) -> i8;
}

impl SongBeatOps for Song {
    fn read_beat(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        voice: &mut Voice,
        start: i64,
        track_index: usize,
    ) -> GpResult<i64> {
        read::read_beat(self, data, seek, voice, start, track_index)
    }
    fn read_beat_v5(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        voice: &mut Voice,
        start: &mut i64,
        track_index: usize,
    ) -> GpResult<i64> {
        read::read_beat_v5(self, data, seek, voice, start, track_index)
    }
    fn read_beat_effects_v3(
        &self,
        data: &[u8],
        seek: &mut usize,
        note_effect: &mut NoteEffect,
    ) -> GpResult<BeatEffects> {
        read::read_beat_effects_v3(self, data, seek, note_effect)
    }
    fn read_beat_effects_v4(&self, data: &[u8], seek: &mut usize) -> GpResult<BeatEffects> {
        read::read_beat_effects_v4(self, data, seek)
    }
    fn read_beat_stroke(&self, data: &[u8], seek: &mut usize) -> GpResult<BeatStroke> {
        read::read_beat_stroke(self, data, seek)
    }
    fn stroke_value(&self, value: i8) -> u8 {
        read::stroke_value(self, value)
    }
    fn read_tremolo_bar(&self, data: &[u8], seek: &mut usize) -> GpResult<BendEffect> {
        read::read_tremolo_bar(self, data, seek)
    }
    fn write_beat_v3(&self, data: &mut Vec<u8>, beat: &Beat) -> GpResult<()> {
        write::write_beat_v3(self, data, beat)
    }
    fn write_beat(
        &self,
        data: &mut Vec<u8>,
        beat: &Beat,
        strings: &[(i8, i8)],
        version: &(u8, u8, u8),
    ) -> GpResult<()> {
        write::write_beat(self, data, beat, strings, version)
    }
    fn write_beat_effect_v3(&self, data: &mut Vec<u8>, beat: &Beat) -> GpResult<()> {
        write::write_beat_effect_v3(self, data, beat)
    }
    fn write_beat_effect_v4(
        &self,
        data: &mut Vec<u8>,
        beat: &Beat,
        version: &(u8, u8, u8),
    ) -> GpResult<()> {
        write::write_beat_effect_v4(self, data, beat, version)
    }
    fn write_tremolo_bar(&self, data: &mut Vec<u8>, bar: &Option<BendEffect>) -> GpResult<()> {
        write::write_tremolo_bar(self, data, bar)
    }
    fn write_beat_stroke(
        &self,
        data: &mut Vec<u8>,
        stroke: &BeatStroke,
        version: &(u8, u8, u8),
    ) -> GpResult<()> {
        write::write_beat_stroke(self, data, stroke, version)
    }
    fn from_stroke_value(value: u8) -> i8 {
        write::from_stroke_value(value)
    }
}
