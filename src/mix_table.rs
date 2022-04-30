use fraction::ToPrimitive;

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
impl Default for WahEffect { fn default() -> Self { WahEffect { value: WAH_EFFECT_NONE, display: false }}}
impl WahEffect {
    pub(crate) fn check_value(value: i8) {
        if !(WAH_EFFECT_OFF <= value && value <= 100) {panic!("Value for a wah effect must be in range from -2 to 100")}
    }
    pub(crate) fn is_on(&self) -> bool {self.value <= 0 && self.value <= 100}
    pub(crate) fn is_off(&self) -> bool {self.value == WAH_EFFECT_OFF}
    pub(crate) fn is_none(&self) -> bool {self.value == WAH_EFFECT_NONE}
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
    pub(crate) fn is_just_wah(&self) -> bool {
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
    /// 
    /// Mix table change was modified to support RSE instruments. It is read as in Guitar Pro 3 and is followed by:
    /// - Wah effect. See :meth:`read_wah_effect()`.
    /// - RSE instrument effect. See :meth:`read_rse_instrument_effect()`.
    pub(crate) fn read_mix_table_change(&mut self, data: &[u8], seek: &mut usize) -> MixTableChange {
        let mut tc = MixTableChange::default();
        self.read_mix_table_change_values(data, seek, &mut tc);
        self.read_mix_table_change_durations(data, seek, &mut tc);
        //println!("read_mix_table_change()");
        if self.version.number >= (4,0,0) {
            let flags = self.read_mix_table_change_flags(data, seek, &mut tc);
            if self.version.number >= (5,0,0) {
                tc.wah = Some(self.read_wah_effect(data, seek, flags));
                self.read_rse_instrument_effect(data, seek, &mut tc.rse);
            }
        }
        tc
    }
    /// Read mix table change values. Mix table change values consist of 7 `signed-byte` and an `int`, which correspond to:
    /// - instrument
    /// - RSE instrument. See `read_rse_instrument()` (GP5).
    /// - volume 
    /// - balance
    /// - chorus
    /// - reverb
    /// - phaser
    /// - tremolo
    /// - Tempo name: `int-byte-size-string` (GP5).
    /// - tempo
    /// 
    /// If signed byte is *-1* then corresponding parameter hasn't changed.
    fn read_mix_table_change_values(&mut self, data: &[u8], seek: &mut usize, mtc: &mut MixTableChange) {
        //instrument
        let b = read_signed_byte(data, seek);
        if b >= 0 {mtc.instrument = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //RSE instrument GP5
        if self.version.number.0 == 5 {mtc.rse = self.read_rse_instrument(data, seek);}
        if self.version.number == (5,0,0)  { *seek += 1; }
        //volume
        let b = read_signed_byte(data, seek);
        if b >= 0 {mtc.volume = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //balance
        let b = read_signed_byte(data, seek);
        if b >= 0 {mtc.balance = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //chorus
        let b = read_signed_byte(data, seek);
        if b >= 0 {mtc.chorus = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //reverb
        let b = read_signed_byte(data, seek);
        if b >= 0 {mtc.reverb = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //phaser
        let b = read_signed_byte(data, seek);
        if b >= 0 {mtc.phaser = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //tremolo
        let b = read_signed_byte(data, seek);
        if b >= 0 {mtc.tremolo = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
        //tempo
        if self.version.number >= (5,0,0) {mtc.tempo_name = read_int_byte_size_string(data, seek);}
        let b = read_int(data, seek);
        if b >= 0 {mtc.tempo = Some(MixTableItem{value: b.to_u8().unwrap(), ..Default::default()});}
    }
    /// Read mix table change durations. Durations are read for each non-null `MixTableItem`. Durations are encoded in `signed-byte`.
    /// 
    /// If tempo did change, then one :ref:`bool` is read. If it's true, then tempo change won't be displayed on the score.
    fn read_mix_table_change_durations(&self, data: &[u8], seek: &mut usize, mtc: &mut MixTableChange) {
        if mtc.volume.is_some()  {mtc.volume.take().unwrap().duration  = read_signed_byte(data, seek).to_u8().unwrap();}
        if mtc.balance.is_some() {mtc.balance.take().unwrap().duration = read_signed_byte(data, seek).to_u8().unwrap();}
        if mtc.chorus.is_some()  {mtc.chorus.take().unwrap().duration  = read_signed_byte(data, seek).to_u8().unwrap();}
        if mtc.reverb.is_some()  {mtc.reverb.take().unwrap().duration  = read_signed_byte(data, seek).to_u8().unwrap();}
        if mtc.phaser.is_some()  {mtc.phaser.take().unwrap().duration  = read_signed_byte(data, seek).to_u8().unwrap();}
        if mtc.tremolo.is_some() {mtc.tremolo.take().unwrap().duration = read_signed_byte(data, seek).to_u8().unwrap();}
        if mtc.tempo.is_some()   {
            let mut t = mtc.tempo.take().unwrap();
            t.duration = read_signed_byte(data, seek).to_u8().unwrap();
            mtc.tempo = Some(t);
            mtc.hide_tempo = false;
            if self.version.number >= (5,0,0) {mtc.hide_tempo = read_bool(data, seek);}
        }
    }

    /// Read mix table change flags (Guitar Pro 4). The meaning of flags:
    /// - *0x01*: change volume for all tracks
    /// - *0x02*: change balance for all tracks
    /// - *0x04*: change chorus for all tracks
    /// - *0x08*: change reverb for all tracks
    /// - *0x10*: change phaser for all tracks
    /// - *0x20*: change tremolo for all tracks
    /// 
    /// In GP5, there is one additional flag:
    /// - *0x40*: use RSE
    /// - *0x80*: show wah-wah
    fn read_mix_table_change_flags(&self, data: &[u8], seek: &mut usize, mtc: &mut MixTableChange) -> i8 {
        let flags = read_signed_byte(data, seek);
        //println!("read_mix_table_change_flags(), flags:  {}", flags);
        if mtc.volume.is_some() {
            let mut e = mtc.volume.take().unwrap();
            e.all_tracks = (flags & 0x01) == 0x01;
            mtc.volume = Some(e);
        }
        if mtc.balance.is_some() {
            let mut e = mtc.balance.take().unwrap();
            e.all_tracks = (flags & 0x01) == 0x01;
            mtc.balance = Some(e);
        }
        if mtc.chorus.is_some() {
            let mut e = mtc.chorus.take().unwrap();
            e.all_tracks = (flags & 0x01) == 0x01;
            mtc.chorus = Some(e);
        }
        if mtc.reverb.is_some() {
            let mut e = mtc.reverb.take().unwrap();
            e.all_tracks = (flags & 0x01) == 0x01;
            mtc.reverb = Some(e);
        }
        if mtc.phaser.is_some() {
            let mut e = mtc.phaser.take().unwrap();
            e.all_tracks = (flags & 0x01) == 0x01;
            mtc.phaser = Some(e);
        }
        if mtc.tremolo.is_some() {
            let mut e = mtc.tremolo.take().unwrap();
            e.all_tracks = (flags & 0x01) == 0x01;
            mtc.tremolo = Some(e);
        }
        if self.version.number >= (5,0,0) {mtc.use_rse = (flags & 0x40) == 0x40;}
        flags
    }

    /// Read wah-wah.
    /// - Wah value: :ref:`signed-byte`. See `WahEffect` for value mapping.
    fn read_wah_effect(&self, data: &[u8], seek: &mut usize, flags: i8) -> WahEffect {WahEffect{value: read_signed_byte(data, seek), display: (flags & -0x80) == -0x80 /*(flags & 0x80) == 0x80*/}}

    pub(crate) fn write_mix_table_change(&self, data: &mut Vec<u8>, mix_table_change: &Option<MixTableChange>) {
        if let Some(mtc) = mix_table_change {
            self.write_mix_table_change_values(data, mtc);
            self.write_mix_table_change_durations(data, mtc);
        }
    }
    fn write_mix_table_change_values(&self, data: &mut Vec<u8>, mix_table_change: &MixTableChange) {
        //instrument
        if let Some(i) = &mix_table_change.instrument {write_signed_byte(data, i.value.to_i8().unwrap());}
        else {write_signed_byte(data, -1);}
        //volume
        if let Some(i) = &mix_table_change.volume {write_signed_byte(data, i.value.to_i8().unwrap());}
        else {write_signed_byte(data, -1);}
        //balance
        if let Some(i) = &mix_table_change.balance {write_signed_byte(data, i.value.to_i8().unwrap());}
        else {write_signed_byte(data, -1);}
        //chorus
        if let Some(i) = &mix_table_change.chorus {write_signed_byte(data, i.value.to_i8().unwrap());}
        else {write_signed_byte(data, -1);}
        //reverb
        if let Some(i) = &mix_table_change.reverb {write_signed_byte(data, i.value.to_i8().unwrap());}
        else {write_signed_byte(data, -1);}
        //phaser
        if let Some(i) = &mix_table_change.phaser {write_signed_byte(data, i.value.to_i8().unwrap());}
        else {write_signed_byte(data, -1);}
        //tremolo
        if let Some(i) = &mix_table_change.tremolo {write_signed_byte(data, i.value.to_i8().unwrap());}
        else {write_signed_byte(data, -1);}
        //tempo
        if let Some(i) = &mix_table_change.tempo {write_signed_byte(data, i.value.to_i8().unwrap());}
        else {write_signed_byte(data, -1);}
    }
    fn write_mix_table_change_durations(&self, data: &mut Vec<u8>, mix_table_change: &MixTableChange) {
        //volume
        if let Some(i) = &mix_table_change.volume {write_signed_byte(data, i.duration.to_i8().unwrap());}
        else {write_signed_byte(data, -1);}
        //balance
        if let Some(i) = &mix_table_change.balance {write_signed_byte(data, i.duration.to_i8().unwrap());}
        else {write_signed_byte(data, -1);}
        //chorus
        if let Some(i) = &mix_table_change.chorus {write_signed_byte(data, i.duration.to_i8().unwrap());}
        else {write_signed_byte(data, -1);}
        //reverb
        if let Some(i) = &mix_table_change.reverb {write_signed_byte(data, i.duration.to_i8().unwrap());}
        else {write_signed_byte(data, -1);}
        //phaser
        if let Some(i) = &mix_table_change.phaser {write_signed_byte(data, i.duration.to_i8().unwrap());}
        else {write_signed_byte(data, -1);}
        //tremolo
        if let Some(i) = &mix_table_change.tremolo {write_signed_byte(data, i.duration.to_i8().unwrap());}
        else {write_signed_byte(data, -1);}
        //tempo
        if let Some(i) = &mix_table_change.tempo {write_signed_byte(data, i.duration.to_i8().unwrap());}
        else {write_signed_byte(data, -1);}
    }
}