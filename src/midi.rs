use crate::io::*;

//MIDI channels

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
    fn default() -> Self { MidiChannel { channel: 0, effect_channel: 1, instrument: 25, volume: 104, balance: 64, chorus: 0, reverb: 0, phaser: 0, tremolo: 0, bank: 0, }}
}
impl MidiChannel {
    pub fn is_percussion_channel(self) -> bool {
        if (self.channel % 16) == DEFAULT_PERCUSSION_CHANNEL {true}
        else {false}
    }
    pub fn set_instrument(mut self, instrument: i32) {
        if instrument == -1 && self.is_percussion_channel() { self.instrument = 0; }
        else {self.instrument = instrument;}
    }

    pub fn get_instrument(self) -> i32 {return self.instrument;}
    pub fn get_instrument_name(&self) -> String {return String::from(CHANNEL_DEFAULT_NAMES[self.instrument as usize]);} //TODO: FIXME: does not seems OK

    /// Read MIDI channels. Guitar Pro format provides 64 channels (4 MIDI ports by 16 hannels), the channels are stored in this order:
    ///`port1/channel1`, `port1/channel2`, ..., `port1/channel16`, `port2/channel1`, ..., `port4/channel16`.
    ///
    /// Each channel has the following form:
    ///
    /// * **Instrument**: `int`
    /// * **Volume**: `byte`
    /// * **Balance**: `byte`
    /// * **Chorus**: `byte`
    /// * **Reverb**: `byte`
    /// * **Phaser**: `byte`
    /// * **Tremolo**: `byte`
    /// * **blank1**: `byte` => Backward compatibility with version 3.0
    /// * **blank2**: `byte` => Backward compatibility with version 3.0
    pub fn read(data: &Vec<u8>, seek: &mut usize, channel: u8) -> MidiChannel {
        let instrument = read_int(data, seek);
        let mut c = MidiChannel::default();
        c.channel = channel; c.effect_channel = channel;
        c.volume = read_signed_byte(data, seek); c.balance = read_signed_byte(data, seek);
        c.chorus = read_signed_byte(data, seek); c.reverb = read_signed_byte(data, seek); c.phaser = read_signed_byte(data, seek); c.tremolo = read_signed_byte(data, seek);
        c.set_instrument(instrument);
        //println!("Channel: {}\t Volume: {}\tBalance: {}\tInstrument={}, {}, {}", c.channel, c.volume, c.balance, instrument, c.get_instrument(), c.get_instrument_name());
        *seek += 2;
        return c;
    }
}