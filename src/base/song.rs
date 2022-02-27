use std::collections::BTreeMap;
use std::collections::HashMap;

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
        key: KeySignature::default()
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
#[derive(Clone)]
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


/*
    FMajorFlat = (-8, 0)
    CMajorFlat = (-7, 0)
    GMajorFlat = (-6, 0)
    DMajorFlat = (-5, 0)
    AMajorFlat = (-4, 0)
    EMajorFlat = (-3, 0)
    BMajorFlat = (-2, 0)
    FMajor = (-1, 0)
    CMajor = (0, 0)
    GMajor = (1, 0)
    DMajor = (2, 0)
    AMajor = (3, 0)
    EMajor = (4, 0)
    BMajor = (5, 0)
    FMajorSharp = (6, 0)
    CMajorSharp = (7, 0)
    GMajorSharp = (8, 0)

    DMinorFlat = (-8, 1)
    AMinorFlat = (-7, 1)
    EMinorFlat = (-6, 1)
    BMinorFlat = (-5, 1)
    FMinor = (-4, 1)
    CMinor = (-3, 1)
    GMinor = (-2, 1)
    DMinor = (-1, 1)
    AMinor = (0, 1)
    EMinor = (1, 1)
    BMinor = (2, 1)
    FMinorSharp = (3, 1)
    CMinorSharp = (4, 1)
    GMinorSharp = (5, 1)
    DMinorSharp = (6, 1)
    AMinorSharp = (7, 1)
    EMinorSharp = (8, 1)
*/

const KEY_F_MAJOR_FLAT: u8 = 0;
const KEY_C_MAJOR_FLAT: u8 = 1;
const KEY_G_MAJOR_FLAT: u8 = 2;
const KEY_D_MAJOR_FLAT: u8 = 3;
const KEY_A_MAJOR_FLAT: u8 = 4;
const KEY_E_MAJOR_FLAT: u8 = 5;
const KEY_B_MAJOR_FLAT: u8 = 6;
const KEY_F_MAJOR: u8 = 7;
const KEY_C_MAJOR: u8 = 8;
const KEY_G_MAJOR: u8 = 9;
const KEY_D_MAJOR: u8 = 10;
const KEY_A_MAJOR: u8 = 11;
const KEY_E_MAJOR: u8 = 12;
const KEY_B_MAJOR: u8 = 13;
const KEY_F_MAJOR_SHARP: u8 = 14;
const KEY_C_MAJOR_SHARP: u8 = 15;
const KEY_G_MAJOR_SHARP: u8 = 16;
const KEY_D_MINOR_FLAT: u8 = 17;
const KEY_A_MINOR_FLAT: u8 = 18;
const KEY_E_MINOR_FLAT: u8 = 19;
const KEY_B_MINOR_FLAT: u8 = 20;
const KEY_F_MINOR: u8 = 21;
const KEY_C_MINOR: u8 = 22;
const KEY_G_MINOR: u8 = 23;
const KEY_D_MINOR: u8 = 24;
const KEY_A_MINOR: u8 = 25;
const KEY_E_MINOR: u8 = 26;
const KEY_B_MINOR: u8 = 27;
const KEY_F_MINOR_SHARP: u8 = 28;
const KEY_C_MINOR_SHARP: u8 = 29;
const KEY_G_MINOR_SHARP: u8 = 30;
const KEY_D_MINOR_SHARP: u8 = 31;
const KEY_A_MINOR_SHARP: u8 = 32;
const KEY_E_MINOR_SHARP: u8 = 33;

#[derive(Clone)]
pub struct KeySignature {
    pub key: i8,
    pub is_minor: bool,
}

impl Default for KeySignature {
    fn default() -> Self { KeySignature {
        key: 0,
        is_minor: false,
    }}
}

impl KeySignature {
    pub fn get_note(self) -> u8 {
        let mut n: u8 = if self.is_minor {29} else {8};
        n += self.key as u8;
        return n;
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
    pub volume: u8,
    pub balance: u8,
    pub chorus: u8,
    pub reverb: u8,
    pub phaser: u8,
    pub tremolo: u8,
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
        if instrument == -1 && ((self.channel % 16) == DEFAULT_PERCUSSION_CHANNEL){
            self.instrument = 0;
        }
        else {self.instrument = instrument;}
    }
}
