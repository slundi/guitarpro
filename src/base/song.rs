use std::collections::BTreeMap;
use std::collections::HashMap;

use fraction::ToPrimitive;

// Struct utility to read file: https://stackoverflow.com/questions/55555538/what-is-the-correct-way-to-read-a-binary-file-in-chunks-of-a-fixed-size-and-stor
#[derive(Clone)]
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
	pub channels: Vec<MidiChannel>,
    pub lyrics: Lyrics,
    pub tempo: i16,
    pub key: KeySignature,

    pub(super) triplet_feel: u8,
    pub(super) current_measure_number: u16,
}

impl Default for Song {
	fn default() -> Self { Song {
		name:String::new(), subtitle: String::new(), artist:String::new(), album: String::new(),
        words: String::new(), author:String::new(), date:String::new(),
        copyright:String::new(), writer:String::new(), transcriber:String::new(), comments:String::new(),
        instructions: String::new(),
		tracks:Vec::new(),
		measure_headers:Vec::new(),
		channels:Vec::with_capacity(64),
        lyrics: Lyrics::default(),
        tempo: 0,
        key: KeySignature::default(),

        triplet_feel:0, current_measure_number: 0,
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
#[derive(Clone)]
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
pub struct TimeSignature {
    pub numerator: i8,
    pub denominator: i8,
    pub beams: Vec<i32>,
}

#[derive(Clone)]
pub struct MeasureHeader {
    pub number: u16,
	pub start: i64,
	pub time_signature: TimeSignature,
	pub tempo: i32,
	pub marker: Marker,
	pub repeat_open: bool,
	pub repeat_alternative: i8,
	pub repeat_close: i8,
	pub triplet_feel: u8,
    /// Tonality of the measure
    pub key_signature: KeySignature,
    pub double_bar: bool,
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
        key_signature: KeySignature::default(),
        double_bar: false,
        marker: Marker::default(),
        time_signature: TimeSignature {numerator: 4, denominator: 0, beams: vec![2, 2, 2, 2]}, //TODO: denominator
    }}
}

pub struct _BeatData {
    current_start: i64,
    voices: Vec<VoiceData>
}
/* INIT:
this.currentStart = measure.getStart();
this.voices = new TGVoiceData[TGBeat.MAX_VOICES];
for(int i = 0 ; i < this.voices.length ; i ++ ) this.voices[i] = new TGVoiceData(measure);
*/


pub const _DURATION_QUARTER_TIME: i64 = 960;
pub const _DURATION_WHOLE: u8 = 1;
pub const _DURATION_HALF: u8 = 2;
pub const DURATION_QUARTER: u8 = 4;
pub const _DURATION_EIGHTH: u8 = 8;
pub const _DURATION_SIXTEENTH: u8 = 16;
pub const _DURATION_THIRTY_SECOND: u8 = 32;
pub const _DURATION_SIXTY_FOURTH: u8 = 64;
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
#[derive(Clone)]
pub struct Track {
    pub number: i32,
	pub offset: i32,
	pub channel: MidiChannel, //pub channel_id: i32,
	pub solo: bool,
	pub mute: bool,
    pub visible: bool,
	pub name: String,
    /// A guitar string with a special tuning.
	pub strings: Vec<(u8, u8)>,
	pub color: i32,
    pub percussion_track: bool,
    pub twelve_stringed_guitar_track: bool,
    pub banjo_track: bool,
    pub port: u8,
    pub fret_count: u8,
    pub indicate_tuning: bool,
    pub use_RSE: bool,
}
impl Default for Track {
    fn default() -> Self { Track {
        number: 1,
        offset: 0,
        channel: MidiChannel::default(), //channel_id: 25,
        solo: false, mute: false, visible: true,
        name: String::from("Track 1"),
        strings: vec![(1, 64), (2, 59), (3, 55), (4, 50), (5, 45), (6, 40)],
        banjo_track: false, twelve_stringed_guitar_track: false, percussion_track: false,
        fret_count: 24,
        color: 0xff0000,
        port: 1,
        indicate_tuning: false,
        use_RSE: false,
        
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
    /// The time resulting with a 64th note and a 3/2 tuplet
    pub min_time: u8,
    //division type
    pub division_enters:u8,
    pub division_times:u8
}

impl Duration {
    fn convert_time(&self, time: u64) -> u64 {
        return time * self.division_times as u64 / self.division_enters as u64;
    }
}

impl Default for Duration {
    fn default() -> Self { Duration {
        value: DURATION_QUARTER, dotted: false, double_dotted: false,
        division_enters:1, division_times:1,
        min_time: 0
    }}
}

/// A *n:m* tuplet.
struct Tuplet {
    enters: u8,
    times: u8,
}
impl Tuplet {
    fn _is_supported(self) -> bool {
        return [(1,1), (3,2), (5,4), (6,4), (7,4), (9,8), (10,8), (11,8), (12,8), (13,8)].contains(&(self.enters, self.times));
    }
    fn _get_time(self) -> u8 {
        let result = fraction::Fraction::new(self.enters, self.times);
        if result.denom().expect("Cannot get fraction denominator") == &1 {1}
        else {result.to_u8().expect("Cannot get fraction result")}
    }
}

/*const KEY_F_MAJOR_FLAT: (i8, bool) = (-8, false);
const KEY_C_MAJOR_FLAT: (i8, bool) = (-7, false);
const KEY_G_MAJOR_FLAT: (i8, bool) = (-6, false);
const KEY_D_MAJOR_FLAT: (i8, bool) = (-5, false);
const KEY_A_MAJOR_FLAT: (i8, bool) = (-4, false);
const KEY_E_MAJOR_FLAT: (i8, bool) = (-3, false);
const KEY_B_MAJOR_FLAT: (i8, bool) = (-2, false);
const KEY_F_MAJOR: (i8, bool) = (-1, false);
const KEY_C_MAJOR: (i8, bool) = (0, false);
const KEY_G_MAJOR: (i8, bool) = (1, false);
const KEY_D_MAJOR: (i8, bool) = (2, false);
const KEY_A_MAJOR: (i8, bool) = (3, false);
const KEY_E_MAJOR: (i8, bool) = (4, false);
const KEY_B_MAJOR: (i8, bool) = (5, false);
const KEY_F_MAJOR_SHARP: (i8, bool) = (6, false);
const KEY_C_MAJOR_SHARP: (i8, bool) = (7, false);
const KEY_G_MAJOR_SHARP: (i8, bool) = (8, false);
const KEY_D_MINOR_FLAT: (i8, bool) = (-8, true);
const KEY_A_MINOR_FLAT: (i8, bool) = (-7, true);
const KEY_E_MINOR_FLAT: (i8, bool) = (-6, true);
const KEY_B_MINOR_FLAT: (i8, bool) = (-5, true);
const KEY_F_MINOR: (i8, bool) = (-4, true);
const KEY_C_MINOR: (i8, bool) = (-3, true);
const KEY_G_MINOR: (i8, bool) = (-2, true);
const KEY_D_MINOR: (i8, bool) = (-1, true);
const KEY_A_MINOR: (i8, bool) = (0, true);
const KEY_E_MINOR: (i8, bool) = (1, true);
const KEY_B_MINOR: (i8, bool) = (2, true);
const KEY_F_MINOR_SHARP: (i8, bool) = (3, true);
const KEY_C_MINOR_SHARP: (i8, bool) = (4, true);
const KEY_G_MINOR_SHARP: (i8, bool) = (5, true);
const KEY_D_MINOR_SHARP: (i8, bool) = (6, true);
const KEY_A_MINOR_SHARP: (i8, bool) = (7, true);
const KEY_E_MINOR_SHARP: (i8, bool) = (8, true);*/

pub const KEY_SIGNATURES: [&'static str; 34] = ["F♭ major", "C♭ major", "G♭ major", "D♭ major", "A♭ major", "E♭ major", "B♭ major",
            "F major", "C major", "G major", "D major", "A major", "E major", "B major",
            "F# major", "C# major", "G# major",
            "D♭ minor", "A♭ minor", "E♭ minor", "B♭ minor",
            "F minor", "C minor", "G minor", "D minor", "A minor", "E minor", "B minor",
            "F# minor", "C# minor", "G# minor", "D# minor", "A# minor", "E# minor"];

#[derive(Clone)]
pub struct KeySignature {
    pub key: i8,
    pub is_minor: bool,
}
impl Default for KeySignature { fn default() -> Self { KeySignature { key: 0, is_minor: false, }} }
impl KeySignature {
    pub fn to_string(&self) -> String {
        let index: usize = if self.is_minor {(23i8 + self.key) as usize} else {(8i8 + self.key) as usize};
        return String::from(KEY_SIGNATURES[index]);
    }
}

//MIDI channels
pub const DEFAULT_PERCUSSION_CHANNEL: u8 = 9;
/// A MIDI channel describes playing data for a track.
#[derive(Copy, Clone)]
pub struct MidiChannel {
    pub channel: u8,
    pub effect_channel: u8,
    instrument: i32,
    pub volume: i8,
    pub balance: i8,
    pub chorus: i8,
    pub reverb: i8,
    pub phaser: i8,
    pub tremolo: i8,
    pub bank: i32,
}

impl Default for MidiChannel {
    fn default() -> Self { MidiChannel {
        channel: 0, effect_channel: 0, instrument: 0,
        volume: 104, balance: 64,
        chorus: 0, reverb: 0, phaser: 0, tremolo: 0, bank: 0,
    }}
}

impl MidiChannel {
    pub fn is_percussion_channel(self) -> bool {
        if (self.channel % 16) == DEFAULT_PERCUSSION_CHANNEL {true}
        else {false}
    }
    pub fn set_instrument(mut self, instrument: i32) {
        if instrument == -1 && self.is_percussion_channel() {
            self.instrument = 0;
        }
        else {self.instrument = instrument;}
    }

    pub fn get_instrument(self) -> i32 {return self.instrument;}
    pub fn get_instrument_name(&self) -> String {return String::from(CHANNEL_DEFAULT_NAMES[self.instrument as usize]);} //TODO: FIXME: does not seems OK
}

/// A marker annotation for beats.
#[derive(Clone)]
pub struct Marker {
    pub title: String,
    pub color: i32,
}
impl Default for Marker {fn default() -> Self { Marker {title: "Section".to_owned(), color: 0xff0000}}}
