use fraction::ToPrimitive;

use crate::{io::*, gp::*};

//MIDI channels

pub const CHANNEL_DEFAULT_NAMES: [&str; 128] = ["Piano", "Bright Piano", "Electric Grand", "Honky Tonk Piano", "Electric Piano 1", "Electric Piano 2",
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
#[derive(Debug,Copy,Clone)]
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
    pub bank: u8,
}
impl Default for MidiChannel {
    fn default() -> Self { MidiChannel { channel: 0, effect_channel: 1, instrument: 25, volume: 104, balance: 64, chorus: 0, reverb: 0, phaser: 0, tremolo: 0, bank: 0, }}
}
impl MidiChannel {
    pub(crate) fn is_percussion_channel(self) -> bool {
        (self.channel % 16) == DEFAULT_PERCUSSION_CHANNEL
    }
    pub(crate) fn set_instrument(mut self, instrument: i32) {
        if instrument == -1 && self.is_percussion_channel() { self.instrument = 0; }
        else {self.instrument = instrument;}
    }

    pub(crate) fn get_instrument(self) -> i32 {self.instrument}
    pub(crate) fn get_instrument_name(&self) -> String {String::from(CHANNEL_DEFAULT_NAMES[self.instrument.to_usize().unwrap()])} //TODO: FIXME: does not seems OK
}

impl Song{
    /// Read all the MIDI channels
    pub(crate) fn read_midi_channels(&mut self, data: &[u8], seek: &mut usize) { for i in 0u8..64u8 { self.channels.push(self.read_midi_channel(data, seek, i)); } }
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
    pub(crate) fn read_midi_channel(&self, data: &[u8], seek: &mut usize, channel: u8) -> MidiChannel {
        let instrument = read_int(data, seek);
        let mut c = MidiChannel{channel, effect_channel: channel, ..Default::default()};
        c.volume = read_signed_byte(data, seek); c.balance = read_signed_byte(data, seek);
        c.chorus = read_signed_byte(data, seek); c.reverb = read_signed_byte(data, seek); c.phaser = read_signed_byte(data, seek); c.tremolo = read_signed_byte(data, seek);
        c.set_instrument(instrument);
        //println!("Channel: {}\t Volume: {}\tBalance: {}\tInstrument={}, {}, {}", c.channel, c.volume, c.balance, instrument, c.get_instrument(), c.get_instrument_name());
        *seek += 2; //Backward compatibility with version 3.0
        c
    }
}