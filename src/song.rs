
use fraction::ToPrimitive;

use crate::enums::*;
use crate::io::*;
use crate::headers::*;
use crate::track::*;
use crate::key_signature::*;
use crate::lyric::*;
use crate::midi::*;
use crate::rse::*;


// Struct utility to read file: https://stackoverflow.com/questions/55555538/what-is-the-correct-way-to-read-a-binary-file-in-chunks-of-a-fixed-size-and-stor
#[derive(Debug,Clone)]
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
    pub master_effect: RseMasterEffect,

    //Used to read the file
    pub current_measure_number: Option<usize>,
    pub current_track: Option<usize>,
    pub current_voice_number: Option<usize>,
    pub current_beat_number: Option<usize>,
}

impl Default for Song {
	fn default() -> Self { Song {
        version: Version {data: String::with_capacity(30), clipboard: false, number: AppVersion::Version_5_10}, clipboard: None,
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
        current_measure_number: None, current_track: None, current_voice_number: None, current_beat_number: None,

        master_effect: RseMasterEffect::default(),
	}}
}
impl Song {
    /// Read the song.
    /// 
    /// **GP3**: A song consists of score information, triplet feel, tempo, song key, MIDI channels, measure and track count, measure headers, tracks, measures.
    /// - Version: `byte-size-string` of size 30.
    /// - Score information. See `readInfo`.
    /// - Triplet feel: `bool`. If value is true, then triplet feel is set to eigth.
    /// - Tempo: `int`.
    /// - Key: `int`. Key signature of the song.
    /// - MIDI channels. See `readMidiChannels`.
    /// - Number of measures: `int`.
    /// - Number of tracks: `int`.
    /// - Measure headers. See `readMeasureHeaders`.
    /// - Tracks. See `read_tracks()`.
    /// - Measures. See `read_measures()`.
    pub fn read_data(&mut self, data: &Vec<u8>) {
        let mut seek: usize = 0;
        self.read_version(data, &mut seek);
        self.read_meta(data, &mut seek);
        
        if self.version.number == AppVersion::Version_3_00 || self.version.number == AppVersion::Version_4_0x{
            self.triplet_feel = if read_bool(data, &mut seek) {TripletFeel::EIGHTH} else {TripletFeel::NONE};
            //println!("Triplet feel: {}", self.triplet_feel);
            if self.version.number == AppVersion::Version_4_0x {self.lyrics = read_lyrics(data, &mut seek);} //read lyrics
            self.tempo = read_int(data, &mut seek).to_i16().unwrap();
            self.key.key = read_int(data, &mut seek).to_i8().unwrap();
            //println!("Tempo: {} bpm\t\tKey: {}", self.tempo, self.key.to_string());
            if self.version.number == AppVersion::Version_4_0x {read_signed_byte(data, &mut seek);} //octave
            self.read_midi_channels(data, &mut seek);
            let measure_count = read_int(data, &mut seek).to_usize().unwrap();
            let track_count = read_int(data, &mut seek).to_usize().unwrap();
            //println!("Measures count: {}\tTrack count: {}", measure_count, track_count);
            // Read measure headers. The *measures* are written one after another, their number have been specified previously.
            for i in 1..measure_count + 1  {
                //self.current_measure_number = Some(i.to_i16().unwrap());
                self.read_measure_header(data, &mut seek, i);
            }
            //self.current_measure_number = Some(0);
            for i in 0..track_count {self.read_track(data, &mut seek, i);}
            self.read_measures(data, &mut seek);
            if self.version.number == AppVersion::Version_4_0x {} //annotate error reading
        }
        //read GP5 information
        if self.version.number == AppVersion::Version_5_00 || self.version.number == AppVersion::Version_5_10 {
            //self.lyrics = 
            read_lyrics(data, &mut seek);
            /*self.masterEffect = self.readRSEMasterEffect()
            self.pageSetup = self.readPageSetup()
            self.tempoName = self.readIntByteSizeString()
            self.tempo = self.readInt()
            self.hideTempo = self.readBool() if self.versionTuple > (5, 0, 0) else False
            self.key = gp.KeySignature((self.readSignedByte(), 0))
            self.readInt()  # octave
            channels = self.readMidiChannels()
            directions = self.readDirections()
            self.masterEffect.reverb = self.readInt()
            measureCount = self.readInt()
            trackCount = self.readInt()
            with self.annotateErrors('reading'):
                self.readMeasureHeaders(self, measureCount, directions)
                self.readTracks(self, trackCount, channels)
                self.readMeasures(self) */
        }
    }
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
        let nc = read_int(data, seek).to_usize().unwrap(); //notes count
        if nc >0 { for i in 0..nc { self.notice.push(read_int_size_string(data, seek)); println!("  {}\t\t{}",i, self.notice[self.notice.len()-1]); }}
    }

    /* INIT:
    this.currentStart = measure.getStart();
    this.voices = new TGVoiceData[TGBeat.MAX_VOICES];
    for(int i = 0 ; i < this.voices.length ; i ++ ) this.voices[i] = new TGVoiceData(measure);
    */


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
}