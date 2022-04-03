use crate::{gp::*, mix_table::*, effects::*, chord::*};


#[derive(Clone,PartialEq)]
pub enum BeatStatus {Empty, Normal, Rest}

#[derive(Clone,PartialEq)]
pub enum TupletBracket {None, Start, End}

/// Parameters of beat display
#[derive(Clone,PartialEq)]
pub struct BeatDisplay {
    break_beam: bool,
    force_beam: bool,
    beam_direction: VoiceDirection,
    tuple_bracket: TupletBracket,
    break_secondary: u16,
    break_secondary_tuplet: bool,
    force_bracket: bool,
}
impl Default for BeatDisplay { fn default() -> Self { BeatDisplay { break_beam:false, force_beam:false, beam_direction:VoiceDirection::None, tuple_bracket:TupletBracket::None, break_secondary:0, break_secondary_tuplet:false, force_bracket:false }}}

/// A stroke effect for beats.
#[derive(Clone,PartialEq)]
pub struct BeatStroke {
    pub direction: BeatStrokeDirection,
    pub value: u16,
}
impl Default for BeatStroke { fn default() -> Self { BeatStroke { direction: BeatStrokeDirection::None, value: 0 }}}
impl BeatStroke {
    pub fn swap_direction(&self) -> BeatStroke {
        let mut bs = BeatStroke::default();
        if self.direction == BeatStrokeDirection::Up {bs.direction = BeatStrokeDirection::Down}
        else if self.direction == BeatStrokeDirection::Down {bs.direction = BeatStrokeDirection::Up}
        return bs;
    }
}

/// This class contains all beat effects
#[derive(Clone,PartialEq)]
pub struct BeatEffect {
    pub stroke: BeatStroke,
    pub has_rasgueado: bool,
    pub pick_stroke: BeatStrokeDirection,
    pub chord: Option<Chord>,
    pub fade_in: bool,
    pub tremolo_bar: Option<BendEffect>,
    pub mix_table_change: Option<MixTableChange>,
    pub slap_effect: SlapEffect,
    pub vibrato: bool,
}
impl Default for BeatEffect { fn default() -> Self { BeatEffect {
    stroke: BeatStroke::default(),
    has_rasgueado: false,
    pick_stroke: BeatStrokeDirection::None,
    chord: None,
    fade_in: false,
    tremolo_bar: None,
    mix_table_change: None,
    slap_effect: SlapEffect::None,
    vibrato: false,
}}}
impl BeatEffect {
    pub fn is_chord(&self) -> bool {return self.chord.is_some();}
    pub fn is_tremolo_bar(&self) -> bool {return self.tremolo_bar.is_some();}
    pub fn is_slap_effect(&self) -> bool {return self.slap_effect != SlapEffect::None;}
    pub fn has_pick_stroke(&self) -> bool {return self.pick_stroke != BeatStrokeDirection::None;}
    pub fn is_default(&self) -> bool {
        let d = BeatEffect::default();
        return self.stroke == d.stroke &&
            self.has_rasgueado == d.has_rasgueado &&
            self.pick_stroke == d.pick_stroke &&
            self.fade_in == d.fade_in &&
            self.vibrato == d.vibrato &&
            self.tremolo_bar == d.tremolo_bar &&
            self.slap_effect == d.slap_effect;
    }
}