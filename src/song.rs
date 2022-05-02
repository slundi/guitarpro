
use fraction::ToPrimitive;

use crate::enums::*;
use crate::io::*;
use crate::headers::*;
use crate::page::*;
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

    pub page_setup: PageSetup,

    //Used to read the file
    pub current_measure_number: Option<usize>,
    pub current_track: Option<usize>,
    pub current_voice_number: Option<usize>,
    pub current_beat_number: Option<usize>,
}

impl Default for Song {
	fn default() -> Self { Song {
        version: Version {data: String::with_capacity(30), clipboard: false, number: (5,1,0)}, clipboard: None,
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

        triplet_feel: TripletFeel::None,
        current_measure_number: None, current_track: None, current_voice_number: None, current_beat_number: None,

        page_setup: PageSetup::default(),

        master_effect: RseMasterEffect::default(),
	}}
}
impl Song {
    /// Read the song. A song consists of score information, triplet feel, tempo, song key, MIDI channels, measure and track count, measure headers, tracks, measures.
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
    pub fn read_gp3(&mut self, data: &[u8]) {
        let mut seek: usize = 0;
        self.version = read_version_string(data, &mut seek);
        self.read_info(data, &mut seek);
        self.triplet_feel = if read_bool(data, &mut seek) {TripletFeel::Eighth} else {TripletFeel::None};
        //println!("Triplet feel: {}", self.triplet_feel);
        self.tempo = read_int(data, &mut seek).to_i16().unwrap();
        self.key.key = read_int(data, &mut seek).to_i8().unwrap();
        //println!("Tempo: {} bpm\t\tKey: {}", self.tempo, self.key.to_string());
        self.read_midi_channels(data, &mut seek);
        let measure_count = read_int(data, &mut seek).to_usize().unwrap();
        let track_count = read_int(data, &mut seek).to_usize().unwrap();
        //println!("Measures count: {}\tTrack count: {}", measure_count, track_count);
        // Read measure headers. The *measures* are written one after another, their number have been specified previously.
        self.read_measure_headers(data, &mut seek, measure_count);
        self.current_measure_number = Some(0);
        self.read_tracks(data, &mut seek, track_count);
        self.read_measures(data, &mut seek);
    }
    /// Read the song. A song consists of score information, triplet feel, tempo, song key, MIDI channels, measure and track count, measure headers, tracks, measures.
    /// - Version: `byte-size-string` of size 30.
    /// - Score information. See `readInfo`.
    /// - Triplet feel: `bool`. If value is true, then triplet feel is set to eigth.
    /// - Lyrics. See `read_lyrics()`.
    /// - Tempo: `int`.
    /// - Key: `int`. Key signature of the song.
    /// - Octave: `signed-byte`. Reserved for future uses.
    /// - MIDI channels. See `readMidiChannels`.
    /// - Number of measures: `int`.
    /// - Number of tracks: `int`.
    /// - Measure headers. See `readMeasureHeaders`.
    /// - Tracks. See `read_tracks()`.
    /// - Measures. See `read_measures()`.
    pub fn read_gp4(&mut self, data: &[u8]) {
        let mut seek: usize = 0;
        self.version = read_version_string(data, &mut seek);
        self.read_clipboard(data, &mut seek);
        self.read_info(data, &mut seek);
        self.triplet_feel = if read_bool(data, &mut seek) {TripletFeel::Eighth} else {TripletFeel::None};
        //println!("Triplet feel: {}", self.triplet_feel);
        self.lyrics = self.read_lyrics(data, &mut seek); //read lyrics
        self.tempo = read_int(data, &mut seek).to_i16().unwrap();
        self.key.key = read_int(data, &mut seek).to_i8().unwrap();
        //println!("Tempo: {} bpm\t\tKey: {}", self.tempo, self.key.to_string());
        read_signed_byte(data, &mut seek); //octave
        self.read_midi_channels(data, &mut seek);
        let measure_count = read_int(data, &mut seek).to_usize().unwrap();
        let track_count = read_int(data, &mut seek).to_usize().unwrap();
        //println!("Measures count: {}\tTrack count: {}", measure_count, track_count);
        // Read measure headers. The *measures* are written one after another, their number have been specified previously.
        self.read_measure_headers(data, &mut seek, measure_count);
        //self.current_measure_number = Some(0);
        self.read_tracks(data, &mut seek, track_count);
        self.read_measures(data, &mut seek);
    }
    pub fn read_gp5(&mut self, data: &[u8]) {
        let mut seek: usize = 0;
        self.version = read_version_string(data, &mut seek);
        self.read_clipboard(data, &mut seek);
        self.read_info(data, &mut seek);
        self.lyrics = self.read_lyrics(data, &mut seek); //read lyrics
        self.master_effect = self.read_rse_master_effect(data, &mut seek);
        self.read_page_setup(data, &mut seek);
        self.tempo_name = read_int_size_string(data, &mut seek);
        self.tempo = read_int(data, &mut seek).to_i16().unwrap();
        self.hide_tempo = if self.version.number > (5,0,0) {read_bool(data, &mut seek)} else {false};
        self.key.key = read_signed_byte(data, &mut seek);
        read_int(data, &mut seek); //octave
        self.read_midi_channels(data, &mut seek);
        let directions = self.read_directions(data, &mut seek);
        self.master_effect.reverb = read_int(data, &mut seek).to_f32().unwrap();
        let measure_count = read_int(data, &mut seek).to_usize().unwrap();
        let track_count = read_int(data, &mut seek).to_usize().unwrap();
        //println!("{} {} {} {:?}", self.tempo_name, self.tempo, self.hide_tempo, self.key.key); //OK
        println!("Track count: {} \t Measure count: {}", track_count, measure_count); //OK
        self.read_measure_headers_v5(data, &mut seek, measure_count, &directions);
        self.read_tracks_v5(data, &mut seek, track_count);
        println!("read_gp5(), after tracks   \t seek: {}", seek);
        self.read_measures(data, &mut seek);
        println!("read_gp5(), after measures \t seek: {}", seek);
    }

    /// Read information (name, artist, ...)
    fn read_info(&mut self, data: &[u8], seek: &mut usize) {
        self.name        = read_int_byte_size_string(data, seek);//.replace("\r", " ").replace("\n", " ").trim().to_owned();
        self.subtitle    = read_int_byte_size_string(data, seek);
        self.artist      = read_int_byte_size_string(data, seek);
        self.album       = read_int_byte_size_string(data, seek);
        self.words       = read_int_byte_size_string(data, seek); //music
        self.author      = if self.version.number.0 < 5 {self.words.clone()} else {read_int_byte_size_string(data, seek)};
        self.copyright   = read_int_byte_size_string(data, seek);
        self.writer      = read_int_byte_size_string(data, seek); //tabbed by
        self.instructions= read_int_byte_size_string(data, seek); //instructions
        //notices
        let nc = read_int(data, seek).to_usize().unwrap(); //notes count
        if nc > 0 { for i in 0..nc { self.notice.push(read_int_byte_size_string(data, seek)); println!("  {}\t\t{}",i, self.notice[self.notice.len()-1]); }}
    }

    /*pub const _MAX_STRINGS: i32 = 25;
    pub const _MIN_STRINGS: i32 = 1;
    pub const _MAX_OFFSET: i32 = 24;
    pub const _MIN_OFFSET: i32 = -24;*/

    /// Write data to a Vec<u8>, you are free to use the encoded data to write it in a file or in a database or do something else.
    pub fn write(&self, version: (u8,u8,u8), clipboard: Option<bool>) ->Vec<u8> {
        let mut data: Vec<u8> = Vec::with_capacity(8388608); //capacity of 8MB, should be sufficient
        write_version(&mut data, version);
        if clipboard.is_some() && clipboard.unwrap() && version.0 >= 4 {self.write_clipboard(&mut data, &version);}
        self.write_info(&mut data, version);
        if version.0 < 5 {write_bool(&mut data, self.triplet_feel != TripletFeel::None);}
        if version.0 >= 4 {self.write_lyrics(&mut data);}
        if version > (5,0,0) {self.write_rse_master_effect(&mut data);}
        if version.0 >= 5 {
            self.write_page_setup(&mut data);
            write_int_byte_size_string(&mut data, &self.tempo_name);
        }
        write_i32(&mut data, self.tempo.to_i32().unwrap());
        if version > (5,0,0) {write_bool(&mut data, self.hide_tempo);}
        write_i32(&mut data, self.key.key.to_i32().unwrap());

        if version.0 >= 4 {write_signed_byte(&mut data, 0);} //octave
        self.write_midi_channels(&mut data);

        if version.0 == 5 {
            self.write_directions(&mut data);
            self.write_master_reverb(&mut data);
        }

        write_i32(&mut data, self.tracks[0].measures.len().to_i32().unwrap());
        write_i32(&mut data, self.tracks.len().to_i32().unwrap());
        self.write_measure_headers(&mut data, &version);
        self.write_tracks(&mut data);
        self.write_measures(&mut data, &version);
        write_i32(&mut data, 0);
        data
    }
    fn write_info(&self, data: &mut Vec<u8>, version: (u8,u8,u8)) {
        write_int_byte_size_string(data, &self.name);
        write_int_byte_size_string(data, &self.subtitle);
        write_int_byte_size_string(data, &self.artist);
        write_int_byte_size_string(data, &self.album);
        if version.0 < 5 {write_int_byte_size_string(data, &self.pack_author());}
        else {
            write_int_byte_size_string(data, &self.words);
            write_int_byte_size_string(data, &self.author);
        }
        write_int_byte_size_string(data, &self.copyright);
        write_int_byte_size_string(data, &self.writer);
        write_int_byte_size_string(data, &self.instructions);
        write_i32(data, self.notice.len().to_i32().unwrap());
        for i in 0..self.notice.len() {write_int_byte_size_string(data, &self.notice[i]);}
    }
    fn pack_author(&self) -> String {
        if !self.words.is_empty() && !self.author.is_empty() {
            if self.words != self.author {
                let mut s = self.words.clone();
                s.push_str(", ");
                s.push_str(&self.author);
                s
            } else {self.words.clone()}
        } else {
            let mut s = self.words.clone();
            s.push_str(&self.author);
            s
        }
    }
}