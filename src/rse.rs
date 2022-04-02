use crate::gp::Accentuation;

/// Equalizer found in master effect and track effect.
/// 
/// Attribute :attr:`RSEEqualizer.knobs` is a list of values in range from -6.0 to 5.9. Master effect has 10 knobs, track effect has 3
/// knobs. Gain is a value in range from -6.0 to 5.9 which can be found in both master and track effects and is named as "PRE" in Guitar Pro 5.
#[derive(Clone)]
pub struct RseEqualizer {
    pub knobs: Vec<f32>,
    pub gain: f32,
}
impl Default for RseEqualizer {fn default() -> Self { RseEqualizer { knobs: Vec::with_capacity(10), gain:0.0 }}}

/// Master effect as seen in "Score information"
#[derive(Clone)]
pub struct RseMasterEffect {
    pub volume: f32,
    pub reverb: f32,
    pub equalizer: RseEqualizer,
}
impl Default for RseMasterEffect { fn default() -> Self { RseMasterEffect {volume:0.0, reverb:0.0, equalizer:RseEqualizer{knobs:vec![0.0;10], ..Default::default()} }}}

#[derive(Clone)]
pub struct RseInstrument {
    pub instrument: i16,
    pub unknown: i16,
    pub sound_bank: i16,
    pub effect_number: i16,
    pub effect_category: String,
    pub effect: String,
}
impl Default for RseInstrument { fn default() -> Self { RseInstrument { instrument:-1, unknown:-1, sound_bank:-1, effect_number:-1, effect_category:String::new(), effect:String::new()}}}

#[derive(Clone)]
pub struct TrackRse {
    pub instrument: RseInstrument,
    pub equalizer: RseEqualizer,
    pub humanize: u8,
    pub auto_accentuation: Accentuation,
}
impl Default for TrackRse { fn default() -> Self { TrackRse {instrument:RseInstrument::default(), humanize:0, auto_accentuation: Accentuation::None, equalizer:RseEqualizer{knobs:vec![0.0;3], ..Default::default()} }}}
