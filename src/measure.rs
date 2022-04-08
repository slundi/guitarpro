use fraction::ToPrimitive;

use crate::{beat::*, gp::*, key_signature::*, io::*, enums::*};

/// A measure header contains metadata for measures over multiple tracks.
#[derive(Debug,Clone)]
pub struct Measure {
    pub number: usize,
    pub start: i64,
    pub has_double_bar: bool,
    pub key_signature: KeySignature,
    pub time_signature: TimeSignature,
    pub track_index: usize,
    pub header_index: usize,
    pub clef: MeasureClef,
    /// Max voice count is 2
    pub voices: Vec<Voice>, 
    pub line_break: LineBreak,

    /*marker: Optional['Marker'] = None
    isRepeatOpen: bool = False
    repeatAlternative: int = 0
    repeatClose: int = -1
    tripletFeel: TripletFeel = TripletFeel.none
    direction: Optional[DirectionSign] = None
    fromDirection: Optional[DirectionSign] = None*/
}
impl Default for Measure {fn default() -> Self { Measure {
    number: 1,
    start: DURATION_QUARTER_TIME,
    has_double_bar: false,
    key_signature: KeySignature::default(), //Cmajor
    time_signature: TimeSignature::default(),
    track_index: 0,
    header_index: 0,
    clef: MeasureClef::Treble,
    voices: vec![Voice::default(), Voice::default()],
    line_break: LineBreak::None
}}}

impl Song {
    /// Read measures. Measures are written in the following order:
    /// - measure 1/track 1
    /// - measure 1/track 2
    /// - ...
    /// - measure 1/track m
    /// - measure 2/track 1
    /// - measure 2/track 2
    /// - ...
    /// - measure 2/track m
    /// - ...
    /// - measure n/track 1
    /// - measure n/track 2
    /// - ...
    /// - measure n/track m
    pub fn read_measures(&mut self, data: &Vec<u8>, seek: &mut usize) {
        let mut start = DURATION_QUARTER_TIME;
        for h in 0..self.measure_headers.len() {
            self.measure_headers[h].start = start;
            for t in 0..self.tracks.len() {
                self.current_track = Some(t);
                let mut m = Measure{track_index:t, header_index:h, ..Default::default()};
                self.current_measure_number = Some(m.number);
                self.read_measure(data, seek, &mut m, t);
                self.tracks[t].measures.push(m);
            }
            //println!("read_measures(), start: {} \t numerator: {} \t denominator: {} \t length: {}", start, self.measure_headers[h].time_signature.numerator, self.measure_headers[h].time_signature.denominator.value, self.measure_headers[h].length());
            start += self.measure_headers[h].length();
        }
        self.current_track = None;
        self.current_measure_number = None;
    }

    /// Read measure. The measure is written as number of beats followed by sequence of beats.
    fn read_measure(&mut self, data: &Vec<u8>, seek: &mut usize, measure: &mut Measure, track_index: usize) {
        self.current_voice_number = Some(1);
        //read a voice 
        let beats = read_int(data, seek).to_usize().unwrap();
        //println!("read_measure() read_voice(), beat count: {}", beats);
        for i in 0..beats {
            self.current_beat_number = Some(i + 1);
            //println!("read_measure() read_voice(), start: {}", measure.start);
            measure.start += self.read_beat(data, seek, &mut measure.voices[0], measure.start, track_index);
            //println!("read_measure() read_voice(), start: {}", measure.start);
        }
        self.current_beat_number = None;
        //end read a voice
        self.current_voice_number = None;
    }
}
