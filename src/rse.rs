use fraction::ToPrimitive;

use crate::{io::*, gp::*, enums::*, track::*};

/// Equalizer found in master effect and track effect.
/// 
/// Attribute :attr:`RSEEqualizer.knobs` is a list of values in range from -6.0 to 5.9. Master effect has 10 knobs, track effect has 3
/// knobs. Gain is a value in range from -6.0 to 5.9 which can be found in both master and track effects and is named as "PRE" in Guitar Pro 5.
#[derive(Debug,Clone)]
pub struct RseEqualizer {
    pub knobs: Vec<f32>,
    pub gain: f32,
}
impl Default for RseEqualizer {fn default() -> Self { RseEqualizer { knobs: Vec::with_capacity(10), gain:0.0 }}}

/// Master effect as seen in "Score information"
#[derive(Debug,Clone)]
pub struct RseMasterEffect {
    pub volume: f32,
    pub reverb: f32,
    pub equalizer: RseEqualizer,
}
impl Default for RseMasterEffect { fn default() -> Self { RseMasterEffect {volume:0.0, reverb:0.0, equalizer:RseEqualizer{knobs:vec![0.0;10], ..Default::default()} }}}

#[derive(Debug,Clone,PartialEq)]
pub struct RseInstrument {
    pub instrument: i16,
    pub unknown: i16,
    pub sound_bank: i16,
    pub effect_number: i16,
    pub effect_category: String,
    pub effect: String,
}
impl Default for RseInstrument { fn default() -> Self { RseInstrument { instrument:-1, unknown:-1, sound_bank:-1, effect_number:-1, effect_category:String::new(), effect:String::new()}}}

#[derive(Debug,Clone)]
pub struct TrackRse {
    pub instrument: RseInstrument,
    pub equalizer: RseEqualizer,
    pub humanize: u8,
    pub auto_accentuation: Accentuation,
}
impl Default for TrackRse { fn default() -> Self { TrackRse {instrument:RseInstrument::default(), humanize:0, auto_accentuation: Accentuation::None, equalizer:RseEqualizer{knobs:vec![0.0;3], ..Default::default()} }}}

impl Song {
    /// Read RSE master effect. Persistence of RSE master effect was introduced in Guitar Pro 5.1. It is read as:
    /// - Master volume: `int`. Values are in range from 0 to 200.
    /// - 10-band equalizer. See `read_equalizer()`.
    pub(crate) fn read_rse_master_effect(&self, data: &[u8], seek: &mut usize) -> RseMasterEffect {
        let mut me = RseMasterEffect::default();
        if self.version.number > (5,0,0) {
            me.volume = read_int(data, seek).to_f32().unwrap();
            read_int(data, seek); //???
            me.equalizer = self.read_rse_equalizer(data, seek, 11);
            //println!("read_rse_master_effect(): {:?}", me);
        }
        me
    }
    /// Read equalizer values. Equalizers are used in RSE master effect and Track RSE. They consist of *n* `SignedBytes <signed-byte>` for each *n* bands and one `signed-byte` for gain (PRE) fader.
    /// Volume values are stored as opposite to actual value. See `unpack_volume_value()`.
    fn read_rse_equalizer(&self, data: &[u8], seek: &mut usize, knobs: u8) -> RseEqualizer {
        let mut e = RseEqualizer::default();
        for _ in 0..knobs {e.knobs.push(self.unpack_volume_value(read_signed_byte(data, seek)));} //knobs = list(map(self.unpackVolumeValue, self.readSignedByte(count=knobsNumber)))
        e                                                                                         //return gp.RSEEqualizer(knobs=knobs[:-1], gain=knobs[-1])
    }
    /// Unpack equalizer volume value. Equalizer volumes are float but stored as `SignedBytes <signed-byte>`.
    fn unpack_volume_value(&self, value: i8) -> f32 { -value.to_f32().unwrap() / 10.0 }

    /// Read track RSE. In GuitarPro 5.1 track RSE is read as follows:
    /// - Humanize: :`byte`.
    /// - Unknown space: 6 `Ints <int>`.
    /// - RSE instrument. See `readRSEInstrument`.
    /// - 3-band track equalizer. See `read_equalizer()`.
    /// - RSE instrument effect. See `read_rse_instrument_effect()`.
    pub(crate) fn read_track_rse(&mut self, data: &[u8], seek: &mut usize, track: &mut Track) {
        track.rse.humanize = read_byte(data, seek);
        //println!("read_track_rse(), humanize: {} \t\t seek: {}", track.rse.humanize, *seek);
        *seek += 12; //read_int(data, seek); read_int(data, seek); read_int(data, seek);  //??? 4 bytes*3 //*seek += 12;
        *seek += 12; //???
        track.rse.instrument = self.read_rse_instrument(data, seek);
        if self.version.number > (5,0,0) {
            track.rse.equalizer = self.read_rse_equalizer(data, seek, 4);
            self.read_rse_instrument_effect(data, seek, &mut track.rse.instrument);
        }
    }
    /// Read RSE instrument.
    /// - MIDI instrument number: `int`.
    /// - Unknown `int`.
    /// - Sound bank: `int`.
    /// - Effect number: `int`. Vestige of Guitar Pro 5.0 format.
    pub(crate) fn read_rse_instrument(&mut self, data: &[u8], seek: &mut usize) -> RseInstrument {
        let mut instrument = RseInstrument{instrument: read_int(data, seek).to_i16().unwrap(), ..Default::default()};
        instrument.unknown    = read_int(data, seek).to_i16().unwrap(); //??? mostly 1
        instrument.sound_bank = read_int(data, seek).to_i16().unwrap();
        //println!("read_rse_instrument(), instrument: {} {} {} \t\t seek: {}", instrument.instrument, instrument.unknown, instrument.sound_bank, *seek);
        if self.version.number == (5,0,0) {
            instrument.effect_number = read_short(data, seek);
            *seek += 1;
        } else {instrument.effect_number = read_int(data, seek).to_i16().unwrap();}
        //println!("read_rse_instrument(), instrument.effect_number: {} \t\t seek: {}", instrument.effect_number, *seek);
        instrument
    }
    /// Read RSE instrument effect name. This feature was introduced in Guitar Pro 5.1.
    /// - Effect name: `int-byte-size-string`.
    /// - Effect category: `int-byte-size-string`.
    pub(crate) fn read_rse_instrument_effect(&mut self, data: &[u8], seek: &mut usize, instrument: &mut RseInstrument) {
        if self.version.number > (5,0,0) {
            instrument.effect =          read_int_byte_size_string(data, seek);
            instrument.effect_category = read_int_byte_size_string(data, seek);
        }
    }

    pub(crate) fn write_rse_master_effect(&self, data: &mut Vec<u8>) {
        write_i32(data, if self.master_effect.volume == 0.0 {100} else {self.master_effect.volume.ceil().to_i32().unwrap()});
        write_i32(data, 0); //reverb?
        self.write_equalizer(data, &self.master_effect.equalizer);

    }
    fn write_equalizer(&self, data: &mut Vec<u8>, equalizer: &RseEqualizer) {
        for i in 0..equalizer.knobs.len() { write_signed_byte(data, self.pack_volume_value(equalizer.knobs[i])); }
        write_signed_byte(data, self.pack_volume_value(equalizer.gain));
    }
    fn pack_volume_value(&self, value: f32) -> i8 {
        (-value * 10f32).round().to_i8().unwrap() //int(-round(value, 1) * 10)
    }
    pub(crate) fn write_master_reverb(&self, data: &mut Vec<u8>) {
        write_i32(data, self.master_effect.reverb.to_i32().unwrap());
    }

    pub(crate) fn write_track_rse(&self, data: &mut Vec<u8>, rse: &TrackRse, version: &(u8,u8,u8)) {
        write_byte(data, rse.humanize);
        write_i32(data, 0); write_i32(data, 0); write_i32(data, 100);
        write_placeholder_default(data, 12);
        self.write_rse_instrument(data, &rse.instrument, version);
        if version > &(5,0,0) {
            self.write_equalizer(data, &rse.equalizer);
            self.write_rse_instrument_effect(data, &rse.instrument);
        }
    }
    pub(crate) fn write_rse_instrument(&self, data: &mut Vec<u8>, instrument: &RseInstrument, version: &(u8,u8,u8)) {
        write_i32(data, instrument.instrument.to_i32().unwrap());
        write_i32(data, instrument.unknown.to_i32().unwrap());
        write_i32(data, instrument.sound_bank.to_i32().unwrap());
        if version == &(5,0,0) {
            write_i16(data, instrument.effect_number);
            write_placeholder_default(data, 1);
        } else {write_i32(data, instrument.effect_number.to_i32().unwrap());}
    }
    pub(crate) fn write_rse_instrument_effect(&self, data: &mut Vec<u8>, instrument: &RseInstrument) { //version>5.0.0
        write_int_byte_size_string(data, &instrument.effect);
        write_int_byte_size_string(data, &instrument.effect_category);
    }
}
