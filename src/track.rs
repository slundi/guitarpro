use fraction::ToPrimitive;

use crate::{io::*, gp::*, rse::*, measure::*};

/// Settings of the track.
#[derive(Debug,Clone)]
pub struct TrackSettings {
    pub tablature: bool,
    pub notation: bool,
    pub diagram_are_below: bool,
    pub show_rythm: bool,
    pub force_horizontal: bool,
    pub force_channels: bool,
    pub diagram_list: bool,
    pub diagram_in_score: bool,
    pub auto_let_ring: bool,
    pub auto_brush: bool,
    pub extend_rythmic: bool,
}
impl Default for TrackSettings { fn default() -> Self { TrackSettings {
    tablature: true,
    notation: true,
    diagram_are_below: false,
    show_rythm: false,
    force_horizontal: false,
    force_channels: false,
    diagram_list: true,
    diagram_in_score: false,
    auto_let_ring: false,
    auto_brush: false,
    extend_rythmic: false,
}}}


#[derive(Debug,Clone)]
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
    pub measures: Vec<Measure>,
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
        use_rse: false, rse: TrackRse::default(),
        measures: Vec::new(),
    }}
}
impl Song {
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
    pub fn read_track(&mut self, data: &Vec<u8>, seek: &mut usize, _number: usize) {
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
            if string_count.to_i8().unwrap() > i { track.strings.push((i + 1, i_tuning)); }
        }
        //println!("tuning: {:?}", track.strings);
        track.port = read_int(data, seek).to_u8().unwrap();
        // Read MIDI channel. MIDI channel in Guitar Pro is represented by two integers. First is zero-based number of channel, second is zero-based number of channel used for effects.
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
        println!("\tInstrument: {} \t Strings: {}/{} ({:?})", self.channels[index].get_instrument_name(), string_count, track.strings.len(), track.strings);
        self.tracks.push(track);
    }
}
