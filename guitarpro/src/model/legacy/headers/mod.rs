mod io;
pub use io::SongHeaderOps;
#[cfg(test)]
pub(crate) use io::{EndingWalks, ending_ordinal};

use crate::error::{GpResult, ToPrimitiveGp};
use crate::model::{
    endings::ending_mask_numbers,
    legacy::{enums::*, key_signature::*},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub data: String,
    pub number: (u8, u8, u8),
    pub clipboard: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clipboard {
    pub start_measure: i32,
    pub stop_measure: i32,
    pub start_track: i32,
    pub stop_track: i32,
    pub start_beat: i32,
    pub stop_beat: i32,
    pub sub_bar_copy: bool,
}
impl Default for Clipboard {
    fn default() -> Self {
        Clipboard {
            start_measure: 1,
            stop_measure: 1,
            start_track: 1,
            stop_track: 1,
            start_beat: 1,
            stop_beat: 1,
            sub_bar_copy: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MeasureHeader {
    pub number: u16,
    pub start: i64,
    pub time_signature: TimeSignature,
    pub tempo: i32,
    pub marker: Option<Marker>,
    pub repeat_open: bool,
    pub repeat_alternative: u8,
    pub repeat_close: i8,
    pub triplet_feel: TripletFeel,
    /// Its direction signs, in [`DirectionSign`] order.
    pub directions: Vec<DirectionSign>,
    /// Tonality of the measure
    pub key_signature: KeySignature,
    pub double_bar: bool,
    /// Fermatas from GPIF (GP6/GP7)
    pub fermatas: Vec<(String, String)>,
    /// Free time (no metronome) from GPIF (GP6/GP7)
    pub free_time: bool,
}
impl Default for MeasureHeader {
    fn default() -> Self {
        MeasureHeader {
            number: 1,
            start: DURATION_QUARTER_TIME,
            tempo: 0,
            repeat_open: false,
            repeat_alternative: 0,
            repeat_close: -1,
            triplet_feel: TripletFeel::None,
            directions: Vec::new(),
            key_signature: KeySignature::default(),
            double_bar: false,
            marker: None,
            time_signature: TimeSignature {
                numerator: 4,
                denominator: Duration::default(),
                beams: vec![2, 2, 2, 2],
            },
            fermatas: Vec::new(),
            free_time: false,
        }
    }
}
impl MeasureHeader {
    #[allow(dead_code)]
    pub(crate) fn length(&self) -> GpResult<i64> {
        Ok(self
            .time_signature
            .numerator
            .to_i64_gp("time_signature.numerator")?
            * crate::model::legacy::key_signature::DURATION_QUARTER_TIME
            * 4
            / self
                .time_signature
                .denominator
                .value
                .to_i64_gp("time_signature.denominator.value")?)
    }
    pub(crate) fn _end(&self) -> GpResult<i64> {
        Ok(self.start + self.length()?)
    }

    /// The 1-based numbers of the alternate endings it's in.
    pub fn ending_numbers(&self) -> Vec<u8> {
        ending_mask_numbers(self.repeat_alternative)
    }

    /// All of its direction signs' labels.
    pub fn direction_labels(&self) -> Vec<String> {
        self.directions
            .iter()
            .map(|sign| sign.label().to_owned())
            .collect()
    }

    /// Adds direction signs, keeping [`MeasureHeader::directions`] in
    /// [`DirectionSign`] order and each sign once.
    pub fn add_directions(&mut self, signs: impl IntoIterator<Item = DirectionSign>) {
        self.directions.extend(signs);
        self.directions.sort();
        self.directions.dedup();
    }
}

/// A marker annotation for beats.
#[derive(Debug, Clone)]
pub struct Marker {
    pub title: String,
    pub color: i32,
}
impl Default for Marker {
    fn default() -> Self {
        Marker {
            title: "Section".to_owned(),
            color: 0xff0000,
        }
    }
}

pub(crate) fn read_marker(data: &[u8], seek: &mut usize) -> GpResult<Marker> {
    use crate::io::primitive::*;
    let mut marker = Marker {
        title: read_int_size_string(data, seek)?,
        ..Default::default()
    };
    marker.color = read_color(data, seek)?;
    Ok(marker)
}

/// This class can store the information about a group of measures which are repeated.
#[derive(Debug, Clone, Default)]
pub struct RepeatGroup {
    pub measure_headers: Vec<usize>,
    pub closings: Vec<usize>,
    pub openings: Vec<usize>,
    pub is_closed: bool,
}
