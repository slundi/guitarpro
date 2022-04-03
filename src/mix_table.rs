use fraction::ToPrimitive;

use crate::rse::*;
use crate::io::*;

/// A mix table item describes a mix parameter, e.g. volume or reverb
#[derive(Clone,PartialEq)]
pub struct MixTableItem {
    pub value: u8,
    pub duration: u8,
    pub all_tracks: bool,
}
impl Default for MixTableItem { fn default() -> Self { MixTableItem { value: 0, duration: 0, all_tracks: false }}}

const WAH_EFFECT_OFF:  i8 = -2;
const WAH_EFFECT_NONE: i8 = -1;
#[derive(Clone,PartialEq)]
pub struct WahEffect {
    value: i8,
    display: bool,
}
impl Default for WahEffect { fn default() -> Self { WahEffect { value:-1, display: false }}}
impl WahEffect {
    pub fn check_value(value: i8) {
        if !(-2 <= value && value <= 100) {panic!("Value for a wah effect must be in range from -2 to 100")}
    }
    pub fn is_on(&self) -> bool {return self.value <= 0 && self.value <= 100;}
    pub fn is_off(&self) -> bool {return self.value == WAH_EFFECT_OFF;}
    pub fn is_none(&self) -> bool {return self.value == WAH_EFFECT_NONE;}
}

/// A MixTableChange describes a change in mix parameters
#[derive(Clone,PartialEq)]
pub struct MixTableChange {
    pub instrument: Option<MixTableItem>,
    pub rse: RseInstrument,
    pub volume: Option<MixTableItem>,
    pub balance: Option<MixTableItem>,
    pub chorus: Option<MixTableItem>,
    pub reverb: Option<MixTableItem>,
    pub phaser: Option<MixTableItem>,
    pub tremolo: Option<MixTableItem>,
    pub tempo_name: String,
    pub tempo: Option<MixTableItem>,
    pub hide_tempo: bool,
    pub wah: Option<WahEffect>,
    pub use_rse: bool,
}
impl Default for MixTableChange { fn default() -> Self { MixTableChange { instrument:None, rse:RseInstrument::default(), volume:None, balance:None, chorus:None, reverb:None, phaser:None, tremolo:None,
        tempo_name:String::new(), tempo:None, hide_tempo:true, wah:None, use_rse:false, 
}}}
impl MixTableChange {
    pub fn is_just_wah(&self) -> bool {
        return self.instrument.is_none() &&  self.volume.is_none() && self.balance.is_none() && self.chorus.is_none() && self.reverb.is_none() && self.phaser.is_none() && self.tremolo.is_none() && self.tempo.is_none() && self.wah.is_none();
    }
    /// Read mix table change. List of values is read first. See `read_values()`.
    /// 
    /// List of values is followed by the list of durations for parameters that have changed. See `read_durations()`.
    pub fn read(data: &Vec<u8>, seek: &mut usize) -> MixTableChange {
        let mut tc = MixTableChange::default();
        tc.read_values(data, seek);
        tc.read_durations(data, seek);
        return tc;
    }
    /// Read mix table change values. Mix table change values consist of 7 `signed-byte` and an `int`, which correspond to:
    /// - instrument
    /// - volume 
    /// - balance
    /// - chorus
    /// - reverb
    /// - phaser
    /// - tremolo
    /// - tempo
    /// 
    /// If signed byte is *-1* then corresponding parameter hasn't changed.
    fn read_values(&mut self, data: &Vec<u8>, seek: &mut usize) {
        //instrument
        let b = read_signed_byte(data, seek);
        if b >= 0 {self.instrument = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //volume
        let b = read_signed_byte(data, seek);
        if b >= 0 {self.volume = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //balance
        let b = read_signed_byte(data, seek);
        if b >= 0 {self.balance = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //chorus
        let b = read_signed_byte(data, seek);
        if b >= 0 {self.chorus = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //reverb
        let b = read_signed_byte(data, seek);
        if b >= 0 {self.reverb = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //phaser
        let b = read_signed_byte(data, seek);
        if b >= 0 {self.phaser = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //tremolo
        let b = read_signed_byte(data, seek);
        if b >= 0 {self.tremolo = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //tempo
        let b = read_signed_byte(data, seek);
        if b >= 0 {self.tempo = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
    }
    /// Read mix table change durations. Durations are read for each non-null `MixTableItem`. Durations are encoded in `signed-byte`.
    fn read_durations(&mut self, data: &Vec<u8>, seek: &mut usize) {
        if self.volume.is_some()  {self.volume.take().unwrap().duration  = read_signed_byte(data, seek).to_u8().unwrap();}
        if self.balance.is_some() {self.balance.take().unwrap().duration = read_signed_byte(data, seek).to_u8().unwrap();}
        if self.chorus.is_some()  {self.chorus.take().unwrap().duration  = read_signed_byte(data, seek).to_u8().unwrap();}
        if self.reverb.is_some()  {self.reverb.take().unwrap().duration  = read_signed_byte(data, seek).to_u8().unwrap();}
        if self.phaser.is_some()  {self.phaser.take().unwrap().duration  = read_signed_byte(data, seek).to_u8().unwrap();}
        if self.tremolo.is_some() {self.tremolo.take().unwrap().duration = read_signed_byte(data, seek).to_u8().unwrap();}
        if self.tempo.is_some()   {
            self.tempo.take().unwrap().duration = read_signed_byte(data, seek).to_u8().unwrap();
            self.hide_tempo = false;
        }
    }
}
