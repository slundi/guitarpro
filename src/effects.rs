/// A single point within the BendEffect
#[derive(Clone)]
pub struct BendPoint {
    pub position: u8,
    pub value: u8,
    pub vibrato: bool,
}
impl Default for BendPoint { fn default() -> Self { BendPoint { position: 0, value: 0, vibrato: false }}}

/// All Bend presets
#[derive(Clone)]
pub enum BendType {
    /// No Preset.
    None,

    //Bends
    /// A simple bend.
    Bend,
    /// A bend and release afterwards.
    BendRelease,
    /// A bend, then release and rebend.
    BendReleaseBend,
    /// Prebend.
    Prebend,
    /// Prebend and then release.
    PrebendRelease,

    //Tremolo Bar
    /// Dip the bar down and then back up.
    Dip,
    /// Dive the bar.
    Dive,
    /// Release the bar up.
    ReleaseUp,
    /// Dip the bar up and then back down.
    InvertedDip,
    /// Return the bar.
    Return,
    /// Release the bar down.
    ReleaseDown
}

/// This effect is used to describe string bends and tremolo bars
#[derive(Clone)]
pub struct BendEffect {
    pub kind: BendType,
    pub value: i32,
    pub points: Vec<BendPoint>,
    /// The note offset per bend point offset
    pub semi_tone_length: u8,
    /// The max position of the bend points (x axis)
    pub max_position: u8,
    /// The max value of the bend points (y axis)
    pub max_value: u8,
}
impl Default for BendEffect { fn default() -> Self { BendEffect { kind: BendType::None, value: 0, points: Vec::with_capacity(12), semi_tone_length: 1, max_position: 12, max_value: 12 /* semi_tone_length * 12 */ }}}

/// All transition types for grace notes
#[derive(Clone)]
pub enum GraceEffectTransition { None, Slide, Bend, Hammer }
pub struct GraceEffect {
    pub duration: u8,
    pub fret: i8,
    pub is_dead: bool,
    pub is_on_beat: bool,
    pub transition: GraceEffectTransition,
    pub velocity: i32,
}
impl Default for GraceEffect {
    fn default() -> Self { GraceEffect {duration: 1, fret: 0, is_dead: false, is_on_beat: false, transition: GraceEffectTransition::None, velocity: 0}} //TODO: velocity
}
