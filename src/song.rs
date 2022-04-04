use fraction::ToPrimitive;

use crate::io::*;
use crate::headers::*;
use crate::key_signature::*;
use crate::beat::*;
use crate::lyric::*;
use crate::midi::*;
use crate::rse::*;


// Struct utility to read file: https://stackoverflow.com/questions/55555538/what-is-the-correct-way-to-read-a-binary-file-in-chunks-of-a-fixed-size-and-stor
#[derive(Clone)]
pub struct Song {
    pub version: Version,
    pub clipboard: Option<Clipboard>,

    pub name: String,
    pub subtitle: String, //Guitar Pro
	pub artist: String,
	pub album: String,
    pub words: String, //GP
	pub author: String, //music by
	pub date: String,
	pub copyright: String,
    /// Tab writer
	pub writer: String,
	pub transcriber: String,
    pub instructions: String,
	pub comments: String,
    pub notice: Vec<String>,

	pub tracks: Vec<Track>,
	pub measure_headers: Vec<MeasureHeader>,
	pub channels: Vec<MidiChannel>,
    pub lyrics: Lyrics,
    pub tempo: i16,
    pub hide_tempo: bool,
    pub tempo_name:String,
    pub key: KeySignature,

    pub triplet_feel: TripletFeel,
    pub current_measure_number: Option<u16>,
    pub current_track: Option<Track>,
    pub master_effect: RseMasterEffect,
}

impl Default for Song {
	fn default() -> Self { Song {
        version: Version {data: String::with_capacity(30), clipboard: false, number: 0}, clipboard: None,
		name:String::new(), subtitle: String::new(), artist:String::new(), album: String::new(),
        words: String::new(), author:String::new(), date:String::new(),
        copyright:String::new(), writer:String::new(), transcriber:String::new(), comments:String::new(),
        notice:Vec::new(),
        instructions: String::new(),
		tracks:Vec::new(),
		measure_headers:Vec::new(),
		channels:Vec::with_capacity(64),
        lyrics: Lyrics::default(),
        tempo: 120, hide_tempo: false, tempo_name:String::from("Moderate"),
        key: KeySignature::default(),

        triplet_feel: TripletFeel::NONE,
        current_measure_number: None, current_track: None,

        master_effect: RseMasterEffect::default(),
	}}
}

impl Song {
    /// Read meta information (name, artist, ...)
    fn read_meta(&mut self, data: &Vec<u8>, seek: &mut usize) {
        // read GP3 informations
        self.name        = read_int_size_string(data, seek);//.replace("\r", " ").replace("\n", " ").trim().to_owned();
        self.subtitle    = read_int_size_string(data, seek);
        self.artist      = read_int_size_string(data, seek);
        self.album       = read_int_size_string(data, seek);
        self.words       = read_int_size_string(data, seek); //music
        self.author      = self.words.clone(); //GP3
        self.copyright   = read_int_size_string(data, seek);
        self.writer      = read_int_size_string(data, seek); //tabbed by
        self.instructions= read_int_size_string(data, seek); //instructions
        //notices
        let nc = read_int(data, seek) as usize; //notes count
        if nc >0 { for i in 0..nc { self.notice.push(read_int_size_string(data, seek)); println!("  {}\t\t{}",i, self.notice[self.notice.len()-1]); }}
    }

    pub fn read_data(&mut self, data: &Vec<u8>) {
        let mut seek: usize = 0;
        read_version(data, &mut seek, self);
        self.read_meta(data, &mut seek);
        
        if self.version.number < VERSION_5_00 {
            self.triplet_feel = if read_bool(data, &mut seek) {TripletFeel::EIGHTH} else {TripletFeel::NONE};
            //println!("Triplet feel: {}", self.triplet_feel);
            if self.version.number == VERSION_4_0X {} //read lyrics
            self.tempo = read_int(data, &mut seek) as i16;
            self.key.key = read_int(data, &mut seek) as i8;
            println!("Tempo: {} bpm\t\tKey: {}", self.tempo, self.key.to_string());
            if self.version.number == VERSION_4_0X {read_signed_byte(data, &mut seek);} //octave
            read_midi_channels(data, &mut seek, &mut self.channels);
            let measure_count = read_int(data, &mut seek) as usize;
            let track_count = read_int(data, &mut seek) as usize;
            println!("Measures count: {}\tTrack count: {}", measure_count, track_count);
            // Read measure headers. The *measures* are written one after another, their number have been specified previously.
            for i in 1..measure_count + 1  {
                //self.current_measure_number = Some(i as u16);
                read_measure_header(data, &mut seek, self, i);
            }
            //self.current_measure_number = Some(0);
            // read tracks //TODO: FIXME
            for i in 0..track_count {self.read_track(data, &mut seek, i);}
            self.read_measures(data, &mut seek);
            if self.version.number == VERSION_4_0X {} //annotate error reading
        }
        //read GP5 information
        if self.version.number == VERSION_5_00 || self.version.number == VERSION_5_10 {
            //self.lyrics = 
            read_lyrics(data, &mut seek);
            /*song.masterEffect = self.readRSEMasterEffect()
            song.pageSetup = self.readPageSetup()
            song.tempoName = self.readIntByteSizeString()
            song.tempo = self.readInt()
            song.hideTempo = self.readBool() if self.versionTuple > (5, 0, 0) else False
            song.key = gp.KeySignature((self.readSignedByte(), 0))
            self.readInt()  # octave
            channels = self.readMidiChannels()
            directions = self.readDirections()
            song.masterEffect.reverb = self.readInt()
            measureCount = self.readInt()
            trackCount = self.readInt()
            with self.annotateErrors('reading'):
                self.readMeasureHeaders(song, measureCount, directions)
                self.readTracks(song, trackCount, channels)
                self.readMeasures(song) */
        }
    }

    /// Read a  track. The first byte is the track's flags. It presides the track's attributes:
    /// 
    /// | **bit 7 to 3** | **bit 2**   | **bit 1**                | **bit 0**   |
    /// |----------------|-------------|--------------------------|-------------|
    /// | Blank bits     | Banjo track | 12 stringed guitar track | Drums track |
    ///
    /// Flags are followed by:
    ///
    /// * **Name**: `string`. A 40 characters long string containing the track's name.
    /// * **Number of strings**: `integer`. An integer equal to the number of strings of the track.
    /// * **Tuning of the strings**: Table of integers. The tuning of the strings is stored as a 7-integers table, the "Number of strings" first integers being really used. The strings are stored from the highest to the lowest.
    /// * **Port**: `integer`. The number of the MIDI port used.
    /// * **Channel**: `integer`. The number of the MIDI channel used. The channel 10 is the drums channel.
    /// * **ChannelE**: `integer`. The number of the MIDI channel used for effects.
    /// * **Number of frets**: `integer`. The number of frets of the instrument.
    /// * **Height of the capo**: `integer`. The number of the fret on which a capo is present. If no capo is used, the value is `0x00000000`.
    /// * **Track's color**: `color`. The track's displayed color in Guitar Pro.
    fn read_track(&mut self, data: &Vec<u8>, seek: &mut usize, _number: usize) {
        let mut track = Track::default();
        //read the flag
        let flags = read_byte(data, seek);
        track.percussion_track = (flags & 0x01) == 0x01; //Drums track
        track.twelve_stringed_guitar_track = (flags & 0x02) == 0x02; //12 stringed guitar track
        track.banjo_track = (flags & 0x04) == 0x04; //Banjo track

        track.name = read_byte_size_string(data, seek);
        *seek += 40 - track.name.len();
        println!("Track: {}", track.name);
        let string_count = read_int(data, seek).to_u8().unwrap();
        track.strings.clear();
        for i in 0i8..7i8 {
            let i_tuning = read_int(data, seek).to_i8().unwrap();
            //println!("tuning: {}", i_tuning);
            if string_count.to_i8().unwrap() > i { track.strings.push((i + 1, i_tuning)); }
        }
        track.port = read_int(data, seek).to_u8().unwrap();
        // Read MIDI channel. MIDI channel in Guitar Pro is represented by two integers. First
        // is zero-based number of channel, second is zero-based number of channel used for effects.
        let index = (read_int(data, seek) -1).to_usize().unwrap();
        let effect_channel = read_int(data, seek) -1;
        if index < self.channels.len() {
            track.channel_index = index;
            if self.channels[index].get_instrument() < 0 {self.channels[index].set_instrument(0);}
            if !self.channels[index].is_percussion_channel() {self.channels[index].effect_channel = effect_channel.to_u8().unwrap();}
        }
        //
        if self.channels[index].channel == 9 {track.percussion_track = true;}
        track.fret_count = read_int(data, seek).to_u8().unwrap();
        track.offset = read_int(data, seek);
        track.color = read_color(data, seek);
        //println!("\tInstrument: {} \t Strings: {} {} ({:?})", track.channel.get_instrument_name(), string_count, track.strings.len(), track.strings);
        self.tracks.push(track);
    }

    fn read_measures(&mut self, data: &Vec<u8>, seek: &mut usize) {
        let mut start = DURATION_QUARTER_TIME;
        for h in 0..self.measure_headers.len() {
            self.measure_headers[h].start = start;
            for t in 0..self.tracks.len() {
                self.current_track = Some(self.tracks[t].clone());
                let mut m = Measure::default();
                m.track_index = t; //measure = gp.Measure(track, header)
                m.header_index= h; //self._currentMeasureNumber = measure.number
                { //Read a measure
                    let start = self.measure_headers[h].start;
                    let voice = Voice::default(); //&m.voices[0];
                    let mut current_voice_number = 1;
                    let mut current_beat_number = 1;
                    { //read_voice
                        let beats = read_int(data, seek).to_usize().unwrap();
                        for b in 0..beats {
                            current_beat_number = b + 1
                            //start += self.readBeat(start, voice)
                            //let flags = read_byte(data, seek);
                            //beat = self.getBeat(voice, start)
                        }
                    }
                    //current_voice_number = None
                }
                //track.measures.append(measure)
            }
            start += self.measure_headers[h].length();
        }
        self.current_track = None;
        self.current_measure_number = None;
    }
    /*fn read_measure(&mut self, data: &Vec<u8>, seek: &mut usize) -> Measure {
        //let mut m = Measure::new();
    }*/
}

/// A navigation sign like *Coda* (𝄌: U+1D10C) or *Segno* (𝄋 or 𝄉: U+1D10B or U+1D109).
#[derive(Clone)]
pub enum DirectionSign { Coda, Segno, }

/// An enumeration of different triplet feels.
#[derive(Clone)]
pub enum TripletFeel { NONE, EIGHTH, SIXTEENTH }

pub struct _BeatData {
    current_start: i64,
    voices: Vec<VoiceData>
}
/* INIT:
this.currentStart = measure.getStart();
this.voices = new TGVoiceData[TGBeat.MAX_VOICES];
for(int i = 0 ; i < this.voices.length ; i ++ ) this.voices[i] = new TGVoiceData(measure);
*/

pub struct VoiceData {
    start: i64,
    velocity: i32,
    flags: i32,
    //duration: Duration
	duration_value: i32,
	duration_dotted: bool,
	duration_double_dotted: bool,
	//duration_division_type: ?
}

/*impl Default for VoiceData {
    fn default() -> Self { VoiceData {
		flags: 0,
		duration_value: DURATION_QUARTER, duration_dotted: false, duration_double_dotted: false
	}}
}*/
/* DEFAUT: 
this.flags = 0;
this.setStart(measure.getStart());
this.setVelocity(TGVelocities.DEFAULT);
*/

pub const _MAX_STRINGS: i32 = 25;
pub const _MIN_STRINGS: i32 = 1;
pub const _MAX_OFFSET: i32 = 24;
pub const _MIN_OFFSET: i32 = -24;

/// Values of auto-accentuation on the beat found in track RSE settings
#[derive(Clone)]
pub enum Accentuation { None, VerySoft, Soft, Medium, Strong, VeryStrong }

#[derive(Clone)]
pub struct Track {
    pub number: i32,
	pub offset: i32,
	pub channel_index: usize, //pub channel_id: i32,
	pub solo: bool,
	pub mute: bool,
    pub visible: bool,
	pub name: String,
    /// A guitar string with a special tuning.
	pub strings: Vec<(i8, i8)>,
	pub color: i32,
    pub percussion_track: bool,
    pub twelve_stringed_guitar_track: bool,
    pub banjo_track: bool,
    pub port: u8,
    pub fret_count: u8,
    pub indicate_tuning: bool,
    pub use_rse: bool,
    pub rse: TrackRse,
}
impl Default for Track {
    fn default() -> Self { Track {
        number: 1,
        offset: 0,
        channel_index: 0, //channel_id: 25,
        solo: false, mute: false, visible: true,
        name: String::from("Track 1"),
        strings: vec![(1, 64), (2, 59), (3, 55), (4, 50), (5, 45), (6, 40)],
        banjo_track: false, twelve_stringed_guitar_track: false, percussion_track: false,
        fret_count: 24,
        color: 0xff0000,
        port: 1,
        indicate_tuning: false,
        use_rse: false, rse: TrackRse::default()
    }}
}

/*
this.number = 0;
this.offset = 0;
this.channelId = -1;
this.solo = false;
this.mute = false;
this.name = new String();
this.measures = new ArrayList<TGMeasure>();
this.strings = new ArrayList<TGString>();
this.color = factory.newColor();
this.lyrics = factory.newLyric();
	public void addMeasure(int index,TGMeasure measure){
		measure.setTrack(this);
		this.measures.add(index,measure);
	}
	
	public TGMeasure getMeasure(int index){
		if(index >= 0 && index < countMeasures()){
			return this.measures.get(index);
		}
		return null;
	}
    public String[] getLyricBeats(){
		String lyrics = getLyrics();
		lyrics = lyrics.replaceAll("\n",REGEX); //REGEX = " "
		lyrics = lyrics.replaceAll("\r",REGEX);
		return lyrics.split(REGEX);
	}
*/

/* 
pub const DEFAULT_PERCUSSION_CHANNEL: i8 = 9;
pub const DEFAULT_PERCUSSION_PROGRAM: i8 = 0;
pub const DEFAULT_PERCUSSION_BANK: i16 = 128;

pub const DEFAULT_BANK: i8 = 0;
pub const DEFAULT_PROGRAM: i8 = 25;
pub const DEFAULT_VOLUME: i8 = 127;
pub const DEFAULT_BALANCE: i8 = 64;
pub const DEFAULT_CHORUS: i8 = 0;
pub const DEFAULT_REVERB: i8 = 0;
pub const DEFAULT_PHASER: i8 = 0;
pub const DEFAULT_TREMOLO: i8 = 0;*/

/// An enumeration of available clefs
#[derive(Clone)]
pub enum MeasureClef { Treble, Bass, Tenor, Alto }
/// A line break directive: `NONE: no line break`, `BREAK: break line`, `Protect the line from breaking`.
#[derive(Clone)]
pub enum LineBreak { None, Break, Protect }
/// Voice directions indicating the direction of beams
#[derive(Clone,PartialEq)]
pub enum VoiceDirection { None, Up, Down }
/// All beat stroke directions
#[derive(Clone,PartialEq)]
pub enum BeatStrokeDirection { None, Up, Down }
#[derive(Clone)]
pub enum BeatStatus { Empty, Normal, Rest }
/// Characteristic of articulation
#[derive(Clone,PartialEq)]
pub enum SlapEffect { None, Tapping, Slapping, Popping }

/// "A measure contains multiple voices of beats
#[derive(Clone)]
pub struct Measure {
    pub track_index: usize,
    pub header_index: usize,
    pub clef: MeasureClef,
    /// Max voice count is 2
    pub voices: Vec<Voice>, 
    pub line_break: LineBreak,
}
impl Default for Measure {fn default() -> Self { Measure {track_index: 0, header_index: 0, clef: MeasureClef::Treble, voices: Vec::with_capacity(2), line_break: LineBreak::None }}}
