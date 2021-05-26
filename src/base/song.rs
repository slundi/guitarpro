use std::collections::BTreeMap;
use std::collections::HashMap;
//use std::io::{self, Read, Seek, SeekFrom};

/*/// Struct utility to read file: https://stackoverflow.com/questions/55555538/what-is-the-correct-way-to-read-a-binary-file-in-chunks-of-a-fixed-size-and-stor
pub struct Chunks<R> {
    pub read: R,
    pub size: usize,
    pub hint: (usize, Option<usize>),
}
impl<R> Chunks<R> {
    pub fn new(read: R, size: usize) -> Self { Self { read, size, hint: (0, None), } }
    pub fn from_seek(mut read: R, size: usize) -> io::Result<Self> where R: Seek, {
        let old_pos = read.seek(SeekFrom::Current(0))?;
        let len = read.seek(SeekFrom::End(0))?;

        let rest = (len - old_pos) as usize; // len is always >= old_pos but they are u64
        if rest != 0 { read.seek(SeekFrom::Start(old_pos))?; }

        let min = rest / size + if rest % size != 0 { 1 } else { 0 };
        Ok(Self { read, size,
            hint: (min, None), // this could be wrong I'm unsure
        })
    }
    // This could be useful if you want to try to recover from an error
    pub fn into_inner(self) -> R { self.read }
}
impl<R> Iterator for Chunks<R> where R: Read, {
    type Item = io::Result<Vec<u8>>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = Vec::with_capacity(self.size);
        match self.read.by_ref().take(chunk.capacity() as u64).read_to_end(&mut chunk) {
            Ok(n) => { if n != 0 { Some(Ok(chunk)) } else {None}}
            Err(e) => Some(Err(e)),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) { self.hint }
}
trait ReadPlus: Read {
    fn chunks(self, size: usize) -> Chunks<Self>
    where Self: Sized, { Chunks::new(self, size) }
}
impl<T: ?Sized> ReadPlus for T where T: Read {}
*/

pub struct Song {
    pub name: String,
	pub artist: String,
	pub album: String,
	pub author: String,
	pub date: String,
	pub copyright: String,
	pub writer: String,
	pub transcriber: String,
	pub comments: String,
	pub tracks: Vec<Track>,
	pub measure_headers: Vec<MeasureHeader>,
	pub channels: Vec<Channel>
}

impl Default for Song {
	fn default() -> Self { Song {
		name:String::new(), artist:String::new(), album: String::new(), author:String::new(), date:String::new(), copyright:String::new(), writer:String::new(), transcriber:String::new(), comments:String::new(),
		tracks:Vec::new(),
		measure_headers:Vec::new(),
		channels:Vec::new(),
	}}
}

const TRIPLET_FEEL_NONE: i32 = 1;
const TRIPLET_FEEL_EIGHTH: i32 = 2;
const TRIPLET_FEEL_SIXTEENTH: i32 = 3;
pub struct MeasureHeader {
    pub number: i32,
	pub start: i64,
	//TGTimeSignature pub time_signature: TimeSignature,
	pub tempo: i32,
	//TGMarker pub marker: Marker,
	pub repeat_pen: bool,
	pub repeat_alternative: i32,
	pub repeat_close: i32,
	pub triplet_feel: i32
	//TGSong song,
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


const DURATION_QUARTER_TIME: i64 = 960;
const DURATION_WHOLE: i32 = 1;
const DURATION_HALF: i32 = 2;
const DURATION_QUARTER: i32 = 4;
const DURATION_EIGHTH: i32 = 8;
const DURATION_SIXTEENTH: i32 = 16;
const DURATION_THIRTY_SECOND: i32 = 32;
const DURATION_SIXTY_FOURTH: i32 = 64;
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

const MAX_STRINGS: i32 = 25;
const MIN_STRINGS: i32 = 1;
const MAX_OFFSET: i32 = 24;
const MIN_OFFSET: i32 = -24;
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
    /// key=from (start at 1), value are the lyrics
	lyrics: BTreeMap<i32, String>,
	//private TGSong song
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

const CHANNEL_DEFAULT_NAMES: [&'static str; 128] = ["Piano", "Bright Piano", "Electric Grand", "Honky Tonk Piano", "Electric Piano 1", "Electric Piano 2",
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

//TODO: handle constants
/* 
const DEFAULT_PERCUSSION_CHANNEL: i8 = 9;
const DEFAULT_PERCUSSION_PROGRAM: i8 = 0;
const DEFAULT_PERCUSSION_BANK: i16 = 128;

const DEFAULT_BANK: i8 = 0;
const DEFAULT_PROGRAM: i8 = 25;
const DEFAULT_VOLUME: i8 = 127;
const DEFAULT_BALANCE: i8 = 64;
const DEFAULT_CHORUS: i8 = 0;
const DEFAULT_REVERB: i8 = 0;
const DEFAULT_PHASER: i8 = 0;
const DEFAULT_TREMOLO: i8 = 0;*/
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

