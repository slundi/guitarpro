use fraction::ToPrimitive;

use crate::{beat::*, gp::*, key_signature::*, io::*, enums::*};

const MAX_VOICES: usize = 2;

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
    voices: Vec::with_capacity(2),
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
    pub(crate) fn read_measures(&mut self, data: &[u8], seek: &mut usize) {
        let mut start = DURATION_QUARTER_TIME;
        for h in 0..self.measure_headers.len() {
            self.measure_headers[h].start = start;
            for t in 0..self.tracks.len() {
                self.current_track = Some(t);
                let mut m = Measure{track_index:t, header_index:h, ..Default::default()};
                self.current_measure_number = Some(m.number);
                if self.version.number < (5,0,0) {self.read_measure(data, seek, &mut m, t);}else {self.read_measure_v5(data, seek, &mut m, t);}
                self.tracks[t].measures.push(m);
            }
            //println!("read_measures(), start: {} \t numerator: {} \t denominator: {} \t length: {}", start, self.measure_headers[h].time_signature.numerator, self.measure_headers[h].time_signature.denominator.value, self.measure_headers[h].length());
            start += self.measure_headers[h].length();
        }
        self.current_track = None;
        self.current_measure_number = None;
    }

    /// Read measure. The measure is written as number of beats followed by sequence of beats.
    fn read_measure(&mut self, data: &[u8], seek: &mut usize, measure: &mut Measure, track_index: usize) {
        //println!("read_measure()");
        let mut voice = Voice::default();
        self.current_voice_number = Some(1);
        self.read_voice(data, seek, &mut voice, &mut measure.start, track_index);
        self.current_voice_number = None;
        measure.voices.push(voice);
        /*
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
        self.current_voice_number = None;*/
    }
    /// Read measure. Guitar Pro 5 stores twice more measures compared to Guitar Pro 3. One measure consists of two sub-measures for each of two voices.
    /// 
    /// Sub-measures are followed by a  `LineBreak` stored in `byte`.
    fn read_measure_v5(&mut self, data: &[u8], seek: &mut usize, measure: &mut Measure, track_index: usize) {
        //println!("read_measure_v5()");
        let mut start = measure.start;
        for number in 0..MAX_VOICES {
            self.current_voice_number = Some(number + 1);
            //println!("read_measure_v5() {:?}",self.current_voice_number);
            let mut voice = Voice::default();
            self.read_voice(data, seek, &mut voice, &mut start, track_index);
            measure.voices.push(voice);
        }
        self.current_voice_number = None;
        if *seek < data.len() {measure.line_break = get_line_break(read_byte(data, seek));} else {measure.line_break = get_line_break(0);}
    }

    fn read_voice(&mut self, data: &[u8], seek: &mut usize, voice: &mut Voice, start: &mut i64, track_index: usize) {
        let beats = read_int(data, seek).to_usize().unwrap();
        for i in 0..beats {
            self.current_beat_number = Some(i + 1);
            //println!("read_measure() read_voice(), start: {}", measure.start);
            *start += if self.version.number < (5,0,0) {self.read_beat(data, seek, voice, *start, track_index)} else {self.read_beat_v5(data, seek, voice, &mut *start, track_index)};
            //println!("read_measure() read_voice(), start: {}", measure.start);
        }
        self.current_beat_number = None;
    }

    pub(crate) fn write_measures(&self, data: &mut Vec<u8>, version: &(u8,u8,u8)) {
        for i in 0..self.tracks.len() {
            //self.current_track = Some(i);
            for m in 0..self.tracks[i].measures.len() {
                //self.current_measure_number = Some(self.tracks[i].measure.number);
                self.write_measure(data, i, m, version);
            }
        }
        //self.current_track = None;
        //self.current_measure_number = None;
    }
    fn write_measure(&self, data: &mut Vec<u8>, track: usize, measure: usize, version: &(u8,u8,u8)) {
        //self.current_voice_number = Some(1);
        self.write_voice(data, track, measure,0, version);
        //self.current_voice_number = None;
    }
    fn write_voice(&self, data: &mut Vec<u8>, track: usize, measure: usize, voice: usize, version: &(u8,u8,u8)) {
        write_i32(data, self.tracks[track].measures[measure].voices[voice].beats.len().to_i32().unwrap());
        for b in 0..self.tracks[track].measures[measure].voices[voice].beats.len() {
            //self.current_beat_number = Some(b+1);
            if version.0 ==3 {self.write_beat_v3(data, &self.tracks[track].measures[measure].voices[voice].beats[b]);}
            else             {self.write_beat(data, &self.tracks[track].measures[measure].voices[voice].beats[b], &self.tracks[track].strings, version);}
            //self.current_beat_number = None;
        }
    }
}
