use fraction::ToPrimitive;

use crate::error::{GpResult, ToPrimitiveGp};
use crate::{
    io::primitive::*,
    model::legacy::{beat::*, enums::*, key_signature::*, song::*},
};

const MAX_VOICES: usize = 2;

/// A measure header contains metadata for measures over multiple tracks.
#[derive(Debug, Clone)]
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
    /// Simile mark from GPIF (GP6/GP7)
    pub simile_mark: Option<String>,
    /*marker: Optional['Marker'] = None
    isRepeatOpen: bool = False
    repeatAlternative: int = 0
    repeatClose: int = -1
    tripletFeel: TripletFeel = TripletFeel.none
    direction: Optional[DirectionSign] = None
    fromDirection: Optional[DirectionSign] = None*/
}
impl Default for Measure {
    fn default() -> Self {
        Measure {
            number: 1,
            start: DURATION_QUARTER_TIME,
            has_double_bar: false,
            key_signature: KeySignature::default(), //Cmajor
            time_signature: TimeSignature::default(),
            track_index: 0,
            header_index: 0,
            clef: MeasureClef::Treble,
            voices: Vec::with_capacity(2),
            line_break: LineBreak::None,
            simile_mark: None,
        }
    }
}

pub trait SongMeasureOps {
    fn read_measures(&mut self, data: &[u8], seek: &mut usize) -> GpResult<()>;
    fn read_measure(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        measure: &mut Measure,
        track_index: usize,
    ) -> GpResult<()>;
    fn read_measure_v5(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        measure: &mut Measure,
        track_index: usize,
    ) -> GpResult<()>;
    fn read_voice(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        voice: &mut Voice,
        start: &mut i64,
        track_index: usize,
    ) -> GpResult<()>;
    fn write_measures(&self, data: &mut Vec<u8>, version: &(u8, u8, u8)) -> GpResult<()>;
    fn write_measure(
        &self,
        data: &mut Vec<u8>,
        track: usize,
        measure: usize,
        version: &(u8, u8, u8),
    ) -> GpResult<()>;
    fn write_voice(
        &self,
        data: &mut Vec<u8>,
        track: usize,
        measure: usize,
        voice: usize,
        version: &(u8, u8, u8),
    ) -> GpResult<()>;
}

impl SongMeasureOps for Song {
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
    fn read_measures(&mut self, data: &[u8], seek: &mut usize) -> GpResult<()> {
        for h in 0..self.measure_headers.len() {
            for t in 0..self.tracks.len() {
                self.current_track = Some(t);
                let mut m = Measure {
                    track_index: t,
                    header_index: h,
                    ..Default::default()
                };
                self.current_measure_number = Some(m.number);
                if self.version.number < (5, 0, 0) {
                    self.read_measure(data, seek, &mut m, t)?;
                } else {
                    self.read_measure_v5(data, seek, &mut m, t)?;
                }
                self.tracks[t].measures.push(m);
            }
            //println!("read_measures(), start: {} \t numerator: {} \t denominator: {} \t length: {}", start, self.measure_headers[h].time_signature.numerator, self.measure_headers[h].time_signature.denominator.value, self.measure_headers[h].length());
        }
        self.current_track = None;
        self.current_measure_number = None;
        for track in &mut self.tracks {
            resolve_tied_frets(track);
        }
        Ok(())
    }

    /// Read measure. The measure is written as number of beats followed by sequence of beats.
    fn read_measure(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        measure: &mut Measure,
        track_index: usize,
    ) -> GpResult<()> {
        //println!("read_measure()");
        let mut voice = Voice::default();
        self.current_voice_number = Some(1);
        self.read_voice(data, seek, &mut voice, &mut measure.start, track_index)?;
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
        Ok(())
    }
    /// Read measure. Guitar Pro 5 stores twice more measures compared to Guitar Pro 3. One measure consists of two sub-measures for each of two voices.
    ///
    /// Sub-measures are followed by a  `LineBreak` stored in `byte`.
    fn read_measure_v5(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        measure: &mut Measure,
        track_index: usize,
    ) -> GpResult<()> {
        //println!("read_measure_v5()");
        let mut start = measure.start;
        for number in 0..MAX_VOICES {
            self.current_voice_number = Some(number + 1);
            //println!("read_measure_v5() {:?}",self.current_voice_number);
            let mut voice = Voice::default();
            self.read_voice(data, seek, &mut voice, &mut start, track_index)?;
            measure.voices.push(voice);
        }
        self.current_voice_number = None;
        if *seek < data.len() {
            measure.line_break = get_line_break(read_byte(data, seek)?);
        } else {
            measure.line_break = get_line_break(0);
        }
        Ok(())
    }

    fn read_voice(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        voice: &mut Voice,
        start: &mut i64,
        track_index: usize,
    ) -> GpResult<()> {
        if *seek + 4 > data.len() {
            return Ok(());
        }
        let beats = read_int(data, seek)?.to_usize().unwrap_or(0);
        //Sanity check
        if beats > 256 {
            return Ok(());
        }
        for i in 0..beats {
            if *seek + 5 > data.len() {
                break;
            }
            self.current_beat_number = Some(i + 1);
            //println!("read_measure() read_voice(), start: {}", measure.start);
            *start += if self.version.number < (5, 0, 0) {
                self.read_beat(data, seek, voice, *start, track_index)?
            } else {
                self.read_beat_v5(data, seek, voice, &mut *start, track_index)?
            };
            //println!("read_measure() read_voice(), start: {}", measure.start);
        }
        self.current_beat_number = None;
        Ok(())
    }

    fn write_measures(&self, data: &mut Vec<u8>, version: &(u8, u8, u8)) -> GpResult<()> {
        for h in 0..self.measure_headers.len() {
            for t in 0..self.tracks.len() {
                self.write_measure(data, t, h, version)?;
            }
        }
        Ok(())
    }
    fn write_measure(
        &self,
        data: &mut Vec<u8>,
        track: usize,
        measure: usize,
        version: &(u8, u8, u8),
    ) -> GpResult<()> {
        //self.current_voice_number = Some(1);
        if version.0 < 5 {
            self.write_voice(data, track, measure, 0, version)?;
        } else {
            // Always emit the fixed voice count the GP5 reader consumes, not
            // however many the in-memory measure happens to hold.
            for v in 0..MAX_VOICES {
                self.write_voice(data, track, measure, v, version)?;
            } //self.current_voice_number = Some(v+1);
            if version.0 == 5 {
                write_byte(
                    data,
                    from_line_break(&self.tracks[track].measures[measure].line_break),
                );
            }
        }
        //self.current_voice_number = None;
        Ok(())
    }
    fn write_voice(
        &self,
        data: &mut Vec<u8>,
        track: usize,
        measure: usize,
        voice: usize,
        version: &(u8, u8, u8),
    ) -> GpResult<()> {
        // A `Song` built by a converter may carry fewer voices than the
        // format reserves (the MusicXML importer emits one voice per XML
        // `<voice>`, often just one). `read_measure_v5` always consumes
        // `MAX_VOICES`, so an absent voice is written as an empty one rather
        // than skipped — otherwise the stream ends early and reading the file
        // back fails with an unexpected EOF.
        let Some(voice_data) = self.tracks[track].measures[measure].voices.get(voice) else {
            write_i32(data, 0);
            return Ok(());
        };

        write_i32(data, voice_data.beats.len().to_i32_gp("beats count")?);
        for beat in &voice_data.beats {
            if version.0 == 3 {
                self.write_beat_v3(data, beat, &self.tracks[track].strings)?;
            } else {
                self.write_beat(data, beat, &self.tracks[track].strings, version)?;
            }
        }
        Ok(())
    }
}

/// Whether a beat's notes go on the [`TieChains`]: not a rest's or an
/// empty beat's. A fixed rule, the same for all three walks.
pub(crate) fn on_tie_chains(status: &BeatStatus) -> bool {
    *status == BeatStatus::Normal
}

/// GP's tie chains: GP marks only a tie's end, tying the note from the
/// last one sounding on its string in its voice (rests, empty beats and
/// rest notes don't count), or with none there, from the last one on that
/// string in an earlier measure, in any voice (its own, of notes starting
/// together in several). The reader, the import into
/// the optimized model and the export from it all pair ties by this rule.
/// `T` is what is kept of each string's last note.
pub(crate) struct TieChains<T> {
    /// Per voice and string: its last note in this measure.
    voices: std::collections::HashMap<u8, std::collections::HashMap<i8, T>>,
    /// Per string: its last notes before this measure (several, starting
    /// together in several voices), by voice.
    earlier: std::collections::HashMap<i8, Vec<(u8, T)>>,
    /// This measure's notes: string, start, voice and what is kept.
    this_measure: Vec<(i8, i64, u8, T)>,
}

impl<T: Clone> TieChains<T> {
    pub(crate) fn new() -> Self {
        TieChains {
            voices: std::collections::HashMap::new(),
            earlier: std::collections::HashMap::new(),
            this_measure: Vec::new(),
        }
    }

    /// Starts the next measure.
    pub(crate) fn next_measure(&mut self) {
        self.voices.clear();
        // The last to sound, not the last voice's: by start, in time order.
        self.this_measure
            .sort_by_key(|&(string, start, _, _)| (string, start));
        let mut notes = self.this_measure.drain(..).peekable();
        while let Some((string, start, voice, kept)) = notes.next() {
            let mut last = vec![(voice, kept)];
            let mut last_start = start;
            while let Some((_, start, voice, kept)) =
                notes.next_if(|&(next, _, _, _)| next == string)
            {
                if start != last_start {
                    last.clear();
                    last_start = start;
                }
                last.push((voice, kept));
            }
            self.earlier.insert(string, last);
        }
    }

    /// The note a tie on `string` in `voice` is from.
    pub(crate) fn previous(&self, voice: u8, string: i8) -> Option<&T> {
        self.voices
            .get(&voice)
            .and_then(|strings| strings.get(&string))
            .or_else(|| {
                let last = self.earlier.get(&string)?;
                // The last added: notes come in order within a voice.
                last.iter()
                    .rev()
                    .find(|(from, _)| *from == voice)
                    .or(last.last())
                    .map(|(_, kept)| kept)
            })
    }

    /// A note sounding on `string` in `voice` (not a rest note) from
    /// `start` (a tick, in any measure-wide frame): it becomes the last
    /// there.
    pub(crate) fn note(&mut self, voice: u8, string: i8, start: i64, kept: T) {
        self.voices
            .entry(voice)
            .or_default()
            .insert(string, kept.clone());
        self.this_measure.push((string, start, voice, kept));
    }

    /// One note of a beat on the chains (see [`on_tie_chains`]), in order:
    /// the pairing rule all three walks share. A rest note isn't on them:
    /// `keep` isn't called. Otherwise `keep` gets the note it's tied from,
    /// if it `ends` a tie and one is there, and gives what is kept of it:
    /// it becomes the last on its string.
    pub(crate) fn link(
        &mut self,
        voice: u8,
        string: i8,
        start: i64,
        rest: bool,
        ends: bool,
        keep: impl FnOnce(Option<T>) -> T,
    ) {
        if rest {
            return;
        }
        let from = if ends {
            self.previous(voice, string).cloned()
        } else {
            None
        };
        self.note(voice, string, start, keep(from));
    }
}

/// GP marks only a tie's end, not its fret: gives each tied note the fret
/// of the note it's tied from ([`TieChains`]); one with nothing before it
/// keeps the file's.
pub(crate) fn resolve_tied_frets(track: &mut crate::model::legacy::track::Track) {
    let mut chains = TieChains::new();
    for measure in &mut track.measures {
        chains.next_measure();
        for (voice, beats) in measure.voices.iter_mut().map(|v| &mut v.beats).enumerate() {
            let voice = voice as u8;
            // Where the voice is: a beat without a start (no reader makes
            // one) starts where the one before it ends, not all at 0.
            let mut at: Option<i64> = None;
            for beat in beats {
                let start = beat.start.or(at).unwrap_or_default();
                // One of unknown length leaves the next where it starts.
                at = Some(start + beat.duration.time().map_or(0, i64::from));
                if !on_tie_chains(&beat.status) {
                    continue;
                }
                for note in &mut beat.notes {
                    let (rest, ends) = (note.kind == NoteType::Rest, note.kind == NoteType::Tie);
                    chains.link(voice, note.string, start, rest, ends, |from| {
                        if let Some(fret) = from {
                            note.value = fret;
                        }
                        note.value
                    });
                }
            }
        }
    }
}
