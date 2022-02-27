use std::collections::BTreeMap;
use std::collections::HashMap;
//use std::io::{self, Read, Seek, SeekFrom};

// Struct utility to read file: https://stackoverflow.com/questions/55555538/what-is-the-correct-way-to-read-a-binary-file-in-chunks-of-a-fixed-size-and-stor


pub struct Song {
    pub name: String,
    pub subtitle: String, //Guitar Pro
	pub artist: String,
	pub album: String,
    pub words: String, //GP
	pub author: String,
	pub date: String,
	pub copyright: String,
	pub writer: String,
	pub transcriber: String,
    pub instructions: String,
	pub comments: String,
	pub tracks: Vec<Track>,
	pub measure_headers: Vec<MeasureHeader>,
	pub channels: Vec<Channel>,
    pub lyrics: Lyrics,
    pub tempo: i16,
}

impl Default for Song {
	fn default() -> Self { Song {
		name:String::new(), subtitle: String::new(), artist:String::new(), album: String::new(),
        words: String::new(), author:String::new(), date:String::new(),
        copyright:String::new(), writer:String::new(), transcriber:String::new(), comments:String::new(),
        instructions: String::new(),
		tracks:Vec::new(),
		measure_headers:Vec::new(),
		channels:Vec::new(),
        lyrics: Lyrics::default(),
        tempo: 0,
	}}
}

/// Struct to keep lyrics
/// On guitar pro files (gp4 or later), you can have 5 lines of lyrics.
/// It is store on a BTreeMap:
/// * the key is the mesure number
/// * the value is the text. Syntax:
///   * " " (spaces or carry returns): separates the syllables of a word
///   * "+": merge two syllables for the same beat
///   * "\[lorem ipsum...\]": hidden text
pub struct Lyrics {
    pub lyrics1: BTreeMap<i32, String>,
    pub lyrics2: BTreeMap<i32, String>,
    pub lyrics3: BTreeMap<i32, String>,
    pub lyrics4: BTreeMap<i32, String>,
    pub lyrics5: BTreeMap<i32, String>,
}

impl Default for Lyrics {
    fn default() -> Self { Lyrics {
        lyrics1: BTreeMap::new(),
        lyrics2: BTreeMap::new(),
        lyrics3: BTreeMap::new(),
        lyrics4: BTreeMap::new(),
        lyrics5: BTreeMap::new(),
    }}
}

pub const TRIPLET_FEEL_NONE: u8 = 0;
pub const TRIPLET_FEEL_EIGHTH: u8 = 1;
pub const TRIPLET_FEEL_SIXTEENTH: u8 = 2;
#[derive(Clone)]
pub struct MeasureHeader {
    pub number: i32,
	pub start: i64,
	//TGTimeSignature pub time_signature: TimeSignature,
	pub tempo: i32,
	//TGMarker pub marker: Marker,
	pub repeat_open: bool,
	pub repeat_alternative: u8,
	pub repeat_close: u16,
	pub triplet_feel: u8
	//TGSong song,
}

impl Default for MeasureHeader {
    fn default() -> Self { MeasureHeader {
        number: 1,
        start: 0,
        tempo: 0,
        repeat_open: false,
        repeat_alternative: 0,
        repeat_close: 0,
        triplet_feel: 0,
    }}
}
/* DEFAULT:
this.number = 0;
this.start = TGDuration.QUARTER_TIME;
this.timeSignature = factory.newTimeSignature();
this.tempo = factory.newTempo();
this.marker = null;
this.tripletFeel = TRIPLET_FEEL_NONE;
this.repeatOpen = false;
this.repeatClose = 0;
this.repeatAlternative = 0;
this.checkMarker();
	public void setNumber(int number) {
		this.number = number;
		this.checkMarker();
	}
    private void checkMarker(){
		if(hasMarker()){
			this.marker.setMeasure(getNumber());
		}
	}
	public long getLength(){
		return getTimeSignature().getNumerator() * getTimeSignature().getDenominator().getTime();
	}
    //tempo
    public long getInMillis(){
		double millis = (60.00 / getValue() * SECOND_IN_MILLIS);
		return (long)millis;
	}
	
	public long getInUSQ(){
		double usq = ((60.00 / getValue() * SECOND_IN_MILLIS) * 1000.00);
		return (long)usq;
	}
	
	public static TGTempo fromUSQ(TGFactory factory,int usq){
		double value = ((60.00 * SECOND_IN_MILLIS) / (usq / 1000.00));
		TGTempo tempo = factory.newTempo();
		tempo.setValue((int)value);
		return tempo;
	}
*/

pub struct BeatData {
    current_start: i64,
    voices: Vec<VoiceData>
}
/* INIT:
this.currentStart = measure.getStart();
this.voices = new TGVoiceData[TGBeat.MAX_VOICES];
for(int i = 0 ; i < this.voices.length ; i ++ ) this.voices[i] = new TGVoiceData(measure);
*/


pub const DURATION_QUARTER_TIME: i64 = 960;
pub const DURATION_WHOLE: u8 = 1;
pub const DURATION_HALF: u8 = 2;
pub const DURATION_QUARTER: u8 = 4;
pub const DURATION_EIGHTH: u8 = 8;
pub const DURATION_SIXTEENTH: u8 = 16;
pub const DURATION_THIRTY_SECOND: u8 = 32;
pub const DURATION_SIXTY_FOURTH: u8 = 64;
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

pub const MAX_STRINGS: i32 = 25;
pub const MIN_STRINGS: i32 = 1;
pub const MAX_OFFSET: i32 = 24;
pub const MIN_OFFSET: i32 = -24;
pub struct Track {
    number: i32,
	offset: i32,
	channel_id: i32,
	solo: bool,
	mute: bool,
	name: String,
	//measures: Vec<Measure>,
	strings: Vec<String>,
	//color: Color,
	//private TGSong song
}
impl Default for Track {
    fn default() -> Self { Track {
        number: 1,
        offset: 0,
        channel_id: 25,
        solo: false,
        mute: false,
        name: String::from("UNDEFINED"),
        strings: Vec::new(),
        //lyrics: BTreeMap::new()
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

pub const CHANNEL_DEFAULT_NAMES: [&'static str; 128] = ["Piano", "Bright Piano", "Electric Grand", "Honky Tonk Piano", "Electric Piano 1", "Electric Piano 2",
                                            "Harpsichord", "Clavinet", "Celesta",
                                            "Glockenspiel",
                                            "Music Box",
                                            "Vibraphone", "Marimba", "Xylophone", "Tubular Bell",
                                            "Dulcimer",
                                            "Hammond Organ", "Perc Organ", "Rock Organ", "Church Organ", "Reed Organ",
                                            "Accordion",
                                            "Harmonica",
                                            "Tango Accordion",
                                            "Nylon Str Guitar", "Steel String Guitar", "Jazz Electric Gtr", "Clean Guitar", "Muted Guitar", "Overdrive Guitar", "Distortion Guitar", "Guitar Harmonics",
                                            "Acoustic Bass", "Fingered Bass", "Picked Bass", "Fretless Bass", "Slap Bass 1", "Slap Bass 2", "Syn Bass 1", "Syn Bass 2",
                                            "Violin", "Viola", "Cello", "Contrabass",
                                            "Tremolo Strings", "Pizzicato Strings",
                                            "Orchestral Harp",
                                            "Timpani",
                                            "Ensemble Strings", "Slow Strings", "Synth Strings 1", "Synth Strings 2",
                                            "Choir Aahs", "Voice Oohs", "Syn Choir",
                                            "Orchestra Hit",
                                            "Trumpet", "Trombone", "Tuba", "Muted Trumpet", "French Horn", "Brass Ensemble", "Syn Brass 1", "Syn Brass 2",
                                            "Soprano Sax", "Alto Sax", "Tenor Sax", "Baritone Sax",
                                            "Oboe", "English Horn", "Bassoon", "Clarinet", "Piccolo", "Flute", "Recorder", "Pan Flute", "Bottle Blow", "Shakuhachi", "Whistle", "Ocarina",
                                            "Syn Square Wave", "Syn Saw Wave", "Syn Calliope", "Syn Chiff", "Syn Charang", "Syn Voice", "Syn Fifths Saw", "Syn Brass and Lead",
                                            "Fantasia", "Warm Pad", "Polysynth", "Space Vox", "Bowed Glass", "Metal Pad", "Halo Pad", "Sweep Pad", "Ice Rain", "Soundtrack", "Crystal", "Atmosphere",
                                            "Brightness", "Goblins", "Echo Drops", "Sci Fi",
                                            "Sitar", "Banjo", "Shamisen", "Koto", "Kalimba",
                                            "Bag Pipe",
                                            "Fiddle",
                                            "Shanai",
                                            "Tinkle Bell",
                                            "Agogo",
                                            "Steel Drums", "Woodblock", "Taiko Drum", "Melodic Tom", "Syn Drum", "Reverse Cymbal",
                                            "Guitar Fret Noise", "Breath Noise",
                                            "Seashore", "Bird", "Telephone", "Helicopter", "Applause", "Gunshot"];

pub struct Channel {
    pub id: u16,
	pub bank: u16,
	pub program: u16,
	pub volume: u16,
	pub balance: u16,
	pub chorus: u16,
	pub reverb: u16,
	pub phaser: u16,
	pub tremolo: u16,
	pub name: String,
    /// Channel parameters (key-value)
	pub parameters: HashMap<String, u32>
}

//TODO: handle pub constants
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
impl Default for Channel {
    fn default() -> Self { Channel {
        id: 1,
        bank: 0,
        program: 25,
        volume: 127,
        balance: 0,
        chorus: 0,
        reverb: 0,
        phaser: 0,
        tremolo: 0,
        name: String::from("UNDEFINED"),
        parameters: HashMap::new()
    }}
}

pub struct Duration {
    pub value:u8,
    pub dotted: bool,
    pub double_dotted:bool,
    //division type
    pub division_enters:u8,
    pub division_times:u8
}
/*	public static final TGDivisionType NORMAL = newDivisionType(1,1);
	public static final TGDivisionType TRIPLET = newDivisionType(3,2);
	public static final TGDivisionType[] ALTERED_DIVISION_TYPES = new TGDivisionType[]{
		newDivisionType(3,2),
		newDivisionType(5,4),
		newDivisionType(6,4),
		newDivisionType(7,4),
		newDivisionType(9,8),
		newDivisionType(10,8),
		newDivisionType(11,8),
		newDivisionType(12,8),
		newDivisionType(13,8),
	};
	 */

impl Duration {
    fn convert_time(&self, time: u64) -> u64 {
        return time * self.division_times as u64 / self.division_enters as u64;
    }
}

impl Default for Duration {
    fn default() -> Self { Duration {
        value: DURATION_QUARTER, dotted: false, double_dotted: false,
        division_enters:1, division_times:1
    }}
}