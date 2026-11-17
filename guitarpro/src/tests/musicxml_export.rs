//! The MusicXML writer's mapping of model values.

use crate::convert::musicxml::from_optimized::make_clef;
use crate::model::optimized::track::Clef;

#[test]
fn octave_clefs_carry_their_octave_change() {
    let double_bass = make_clef(Clef::Bass8vb, None);
    assert_eq!(double_bass.sign, "F");
    assert_eq!(double_bass.clef_octave_change, Some(-1));
    assert_eq!(make_clef(Clef::Treble, None).clef_octave_change, None);
}
