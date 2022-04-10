use fraction::ToPrimitive;

use crate::enums::*;
use crate::rse::*;
use crate::io::*;
use crate::gp::*;

/// A mix table item describes a mix parameter, e.g. volume or reverb
#[derive(Debug,Clone,PartialEq,Default)]
pub struct MixTableItem {
    pub value: u8,
    pub duration: u8,
    pub all_tracks: bool,
}
//impl Default for MixTableItem { fn default() -> Self { MixTableItem { value: 0, duration: 0, all_tracks: false }}}

const WAH_EFFECT_OFF:  i8 = -2;
const WAH_EFFECT_NONE: i8 = -1;
#[derive(Debug,Clone,PartialEq)]
pub struct WahEffect {
    value: i8,
    display: bool,
}
impl Default for WahEffect { fn default() -> Self { WahEffect { value:-1, display: false }}}
impl WahEffect {
    pub fn check_value(value: i8) {
        if !(-2 <= value && value <= 100) {panic!("Value for a wah effect must be in range from -2 to 100")}
    }
    pub fn is_on(&self) -> bool {self.value <= 0 && self.value <= 100}
    pub fn is_off(&self) -> bool {self.value == WAH_EFFECT_OFF}
    pub fn is_none(&self) -> bool {self.value == WAH_EFFECT_NONE}
}

/// A MixTableChange describes a change in mix parameters
#[derive(Debug,Clone,PartialEq)]
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
        self.instrument.is_none() &&  self.volume.is_none() && self.balance.is_none() && self.chorus.is_none() && self.reverb.is_none() && self.phaser.is_none() && self.tremolo.is_none() && self.tempo.is_none() && self.wah.is_none()
    }
}

impl Song {
    /// Read mix table change. List of values is read first. See `read_values()`.
    /// 
    /// List of values is followed by the list of durations for parameters that have changed. See `read_durations()`.
    /// 
    /// Mix table change in Guitar Pro 4 format extends Guitar Pro 3 format. It constists of `values <read_mix_table_change_values()>`,
    /// `durations <read_mix_table_change_durations()>`, and, new to GP3, `flags <read_mix_table_change_flags()>`.
    pub fn read_mix_table_change(&self, data: &[u8], seek: &mut usize) -> MixTableChange {
        let mut tc = MixTableChange::default();
        self.read_mix_table_change_values(data, seek, &mut tc);
        self.read_mix_table_change_durations(data, seek, &mut tc);
        if self.version.number == AppVersion::Version_4_0x {self.read_mix_table_change_flags(data, seek, &mut tc);}
        tc
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
    fn read_mix_table_change_values(&self, data: &[u8], seek: &mut usize, mte: &mut MixTableChange) {
        //instrument
        let b = read_signed_byte(data, seek);
        if b >= 0 {mte.instrument = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //volume
        let b = read_signed_byte(data, seek);
        if b >= 0 {mte.volume = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //balance
        let b = read_signed_byte(data, seek);
        if b >= 0 {mte.balance = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //chorus
        let b = read_signed_byte(data, seek);
        if b >= 0 {mte.chorus = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //reverb
        let b = read_signed_byte(data, seek);
        if b >= 0 {mte.reverb = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //phaser
        let b = read_signed_byte(data, seek);
        if b >= 0 {mte.phaser = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //tremolo
        let b = read_signed_byte(data, seek);
        if b >= 0 {mte.tremolo = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //tempo
        let b = read_int(data, seek);
        if b >= 0 {mte.tempo = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
    }
    /// Read mix table change durations. Durations are read for each non-null `MixTableItem`. Durations are encoded in `signed-byte`.
    fn read_mix_table_change_durations(&self, data: &[u8], seek: &mut usize, mte: &mut MixTableChange) {
        if mte.volume.is_some()  {mte.volume.take().unwrap().duration  = read_signed_byte(data, seek).to_u8().unwrap();}
        if mte.balance.is_some() {mte.balance.take().unwrap().duration = read_signed_byte(data, seek).to_u8().unwrap();}
        if mte.chorus.is_some()  {mte.chorus.take().unwrap().duration  = read_signed_byte(data, seek).to_u8().unwrap();}
        if mte.reverb.is_some()  {mte.reverb.take().unwrap().duration  = read_signed_byte(data, seek).to_u8().unwrap();}
        if mte.phaser.is_some()  {mte.phaser.take().unwrap().duration  = read_signed_byte(data, seek).to_u8().unwrap();}
        if mte.tremolo.is_some() {mte.tremolo.take().unwrap().duration = read_signed_byte(data, seek).to_u8().unwrap();}
        if mte.tempo.is_some()   {
            mte.tempo.take().unwrap().duration = read_signed_byte(data, seek).to_u8().unwrap();
            mte.hide_tempo = false;
        }
    }

    /// Read mix table change flags (Guitar Pro 4). The meaning of flags:
    /// - *0x01*: change volume for all tracks
    /// - *0x02*: change balance for all tracks
    /// - *0x04*: change chorus for all tracks
    /// - *0x08*: change reverb for all tracks
    /// - *0x10*: change phaser for all tracks
    /// - *0x20*: change tremolo for all tracks
    fn read_mix_table_change_flags(&self, data: &[u8], seek: &mut usize, mte: &mut MixTableChange) -> i8 {
        let flags = read_signed_byte(data, seek);
        if mte.volume.is_some() {
            let mut e = mte.volume.take().unwrap();
            e.all_tracks = (flags & 0x01) == 0x01;
            mte.volume = Some(e);
        }
        if mte.balance.is_some() {
            let mut e = mte.balance.take().unwrap();
            e.all_tracks = (flags & 0x01) == 0x01;
            mte.balance = Some(e);
        }
        if mte.chorus.is_some() {
            let mut e = mte.chorus.take().unwrap();
            e.all_tracks = (flags & 0x01) == 0x01;
            mte.chorus = Some(e);
        }
        if mte.reverb.is_some() {
            let mut e = mte.reverb.take().unwrap();
            e.all_tracks = (flags & 0x01) == 0x01;
            mte.reverb = Some(e);
        }
        if mte.phaser.is_some() {
            let mut e = mte.phaser.take().unwrap();
            e.all_tracks = (flags & 0x01) == 0x01;
            mte.phaser = Some(e);
        }
        if mte.tremolo.is_some() {
            let mut e = mte.tremolo.take().unwrap();
            e.all_tracks = (flags & 0x01) == 0x01;
            mte.tremolo = Some(e);
        }
        flags
    }
}