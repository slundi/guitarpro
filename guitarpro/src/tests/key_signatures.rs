//! Key signatures: fifths and modes, and how they're saved.

use crate::convert::optimized::timeline::key_sig_from_fifths;
use crate::model::optimized::note::PitchStep;

#[test]
fn key_signatures_invert_their_fifths_in_every_mode() {
    for mode in [
        "major",
        "minor",
        "dorian",
        "phrygian",
        "lydian",
        "mixolydian",
        "locrian",
    ] {
        for fifths in -7..=7i8 {
            let key_sig = key_sig_from_fifths(fifths, Some(mode));
            assert_eq!(key_sig.fifths(), fifths, "{mode} fifths={fifths}");
            assert_eq!(key_sig.mode.name(), mode);
        }
    }
    // Beyond seven sharps or flats: the enharmonic key, not C.
    assert_eq!(key_sig_from_fifths(8, None).fifths(), -4);
    assert_eq!(key_sig_from_fifths(-8, None).fifths(), 4);
    // D dorian has no sharps or flats.
    let dorian = key_sig_from_fifths(0, Some("dorian"));
    assert_eq!((dorian.root.step, dorian.root.alter), (PitchStep::D, 0));
}

#[test]
fn key_signatures_saved_with_a_major_root_keep_their_signature() -> serde_json::Result<()> {
    use crate::model::optimized::metadata::{KeySignature, Mode};
    use serde_json::{from_str, to_string};

    // D dorian saved before `tonic`: its signature's major tonic, C.
    let old: KeySignature =
        from_str(r#"{"root":{"step":"C","alter":0,"octave":4},"mode":"Dorian"}"#)?;
    assert_eq!(old, KeySignature::from_fifths(0, Mode::Dorian));
    // Minor keys had their own tonic already.
    let minor: KeySignature =
        from_str(r#"{"root":{"step":"A","alter":0,"octave":4},"mode":"Minor"}"#)?;
    assert_eq!(minor, KeySignature::from_fifths(0, Mode::Minor));
    // Written as `tonic`, read back as is.
    let json = to_string(&old)?;
    assert!(json.contains(r#""tonic""#) && !json.contains(r#""root""#));
    assert_eq!(from_str::<KeySignature>(&json)?, old);
    Ok(())
}
