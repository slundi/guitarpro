/// A mix table item describes a mix parameter, e.g. volume or reverb
#[derive(Clone)]
pub struct MixTableItem {
    pub value: u8,
    pub duration: u8,
    pub all_tracks: bool,
}
impl Default for MixTableItem { fn default() -> Self { MixTableItem { value: 0, duration: 0, all_tracks: false }}}

const WAH_EFFECT_OFF:  i8 = -2;
const WAH_EFFECT_NONE: i8 = -1;
#[derive(Clone)]
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
#[derive(Clone)]
pub struct MixTableChange {
    pub instrument: Option<MixTableItem>,
    //TODO: pub rse: RSEInstrument = attr.Factory(RSEInstrument)
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
impl Default for MixTableChange { fn default() -> Self { MixTableChange { instrument:None, /*rse:,*/ volume:None, balance:None, chorus:None, reverb:None, phaser:None, tremolo:None,
        tempo_name:String::new(), tempo:None, hide_tempo:true, wah:None, use_rse:false
}}}
impl MixTableChange {
    pub fn is_just_wah(&self) -> bool {
        return self.instrument.is_none() &&  self.volume.is_none() && self.balance.is_none() && self.chorus.is_none() && self.reverb.is_none() && self.phaser.is_none() && self.tremolo.is_none() && self.tempo.is_none() && self.wah.is_none();
    }
}
