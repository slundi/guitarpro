//! MuseScore's vocabulary for notation, shared by both conversion
//! directions: dynamic subtypes, articulation/ornament symbols, barline
//! styles, jumps and markers.

use crate::model::legacy::enums::DirectionSign;
use crate::model::mscz::{
    MscxBarLine, MscxDynamic, MscxJump, MscxMarker, MscxMeasureMark, mscx::dynamic_letter_glyph,
};
use crate::model::optimized::{
    beat::Dynamic,
    effect::{HairpinEvent, HairpinKind},
    note::{Articulation, Ornament},
    timeline::{BarlineStyle, JumpKind},
};

// ---------------------------------------------------------------------------
// Dynamics
// ---------------------------------------------------------------------------

/// Whether MuseScore 3/4 know a marking by name as a `<Dynamic><subtype>`:
/// every [`Dynamic`] but sfzp, and the single letters. Anything else is
/// written as `other-dynamics` with the marking as text.
fn is_subtype(name: &str) -> bool {
    match Dynamic::from_name(name) {
        Dynamic::SFZP => false,
        Dynamic::Other(_) => matches!(name, "m" | "r" | "s" | "z"),
        _ => true,
    }
}

/// An `other-dynamics` text as MSCX markup, `escape`d: its dynamic words
/// (`sempre p`) in SMuFL glyphs, as MuseScore writes them. Single letters
/// that are words of their own too (`m`, `s`...) stay text.
pub(crate) fn dynamic_text_markup(text: &str, escape: impl Fn(&str) -> String) -> String {
    text.split(' ')
        .map(|word| {
            let glyphs: Option<String> = (word.len() > 1 || matches!(word, "p" | "f"))
                .then_some(word)
                .filter(|word| is_subtype(word))
                .and_then(|word| {
                    word.chars()
                        .map(|letter| {
                            dynamic_letter_glyph(letter).map(|glyph| format!("<sym>{glyph}</sym>"))
                        })
                        .collect()
                });
            glyphs.unwrap_or_else(|| escape(word))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A `<Dynamic>`'s marking: its subtype, or an `other-dynamics` one's text.
pub(crate) fn dynamic_from_mscx(dynamic: &MscxDynamic) -> Option<Dynamic> {
    match dynamic.subtype.as_deref() {
        Some(subtype) if subtype != "other-dynamics" => Some(Dynamic::from_name(subtype)),
        _ => {
            let text = dynamic.text.as_deref().unwrap_or_default().trim();
            (!text.is_empty()).then(|| Dynamic::from_name(text))
        }
    }
}

/// The `<Dynamic>` for a marking, with the playback velocity MuseScore
/// gives its subtype by default.
pub(crate) fn dynamic_to_mscx(dynamic: &Dynamic) -> MscxDynamic {
    let name = dynamic.name();
    if is_subtype(name) {
        MscxDynamic {
            subtype: Some(name.to_string()),
            text: None,
            velocity: dynamic.playback_velocity(),
            voice_only: false,
        }
    } else {
        MscxDynamic {
            subtype: Some("other-dynamics".to_string()),
            text: Some(name.to_string()),
            velocity: None,
            voice_only: false,
        }
    }
}

/// Whether a staff's dynamics each hold for their own voice only, given
/// the voice of each (in any order): when they are in several voices,
/// which the model gives levels of their own. The reader and the writer
/// both go by this.
pub(crate) fn dynamics_voice_only(mut voices: impl Iterator<Item = u8>) -> bool {
    voices
        .next()
        .is_some_and(|first| voices.any(|voice| voice != first))
}

// ---------------------------------------------------------------------------
// Hairpins
// ---------------------------------------------------------------------------

/// `<HairPin><subtype>`: 0/1 the hairpin shapes, 2/3 the equivalent
/// "cresc."/"dim." lines (the kind, and whether it's a line).
pub(crate) fn hairpin_from_subtype(subtype: u8) -> (HairpinKind, bool) {
    let kind = match subtype {
        1 | 3 => HairpinKind::DecrescendoStart,
        _ => HairpinKind::CrescendoStart,
    };
    (kind, matches!(subtype, 2 | 3))
}

/// Inverse of [`hairpin_from_subtype`]; `None` for a stop.
pub(crate) fn hairpin_subtype(hairpin: &HairpinEvent) -> Option<u8> {
    let line = if hairpin.line { 2 } else { 0 };
    match hairpin.kind {
        HairpinKind::CrescendoStart => Some(line),
        HairpinKind::DecrescendoStart => Some(line + 1),
        HairpinKind::Stop => None,
    }
}

// ---------------------------------------------------------------------------
// Articulations and ornaments
// ---------------------------------------------------------------------------

/// What an `<Articulation>`/`<Ornament>` symbol in a `<Chord>` means.
/// MuseScore 3 stores ornaments (and, in old files, fermatas) as
/// articulation symbols too, so both elements share this.
pub(crate) enum ChordMark {
    Articulation(Articulation),
    Ornament(Ornament),
    Fermata,
}

pub(crate) fn chord_marks_from_symbol(symbol: &str) -> Vec<ChordMark> {
    use Articulation as A;
    let symbol = symbol.trim();
    if symbol.starts_with("fermata") {
        return vec![ChordMark::Fermata];
    }
    if symbol.starts_with("ornament") {
        return vec![ChordMark::Ornament(ornament_from_symbol(symbol))];
    }
    let base = symbol
        .strip_suffix("Above")
        .or_else(|| symbol.strip_suffix("Below"))
        .unwrap_or(symbol);
    let articulations = match base {
        "articAccent" => vec![A::Accent],
        "articMarcato" => vec![A::Marcato],
        "articStaccato" => vec![A::Staccato],
        "articTenuto" => vec![A::Tenuto],
        "articTenutoStaccato" => vec![A::DetachedLegato],
        "articStaccatissimo" | "articStaccatissimoWedge" => vec![A::Staccatissimo],
        "articStaccatissimoStroke" => vec![A::Spiccato],
        "articStress" => vec![A::Stress],
        "articUnstress" => vec![A::Unstress],
        "articSoftAccent" => vec![A::SoftAccent],
        "articAccentStaccato" => vec![A::Accent, A::Staccato],
        "articMarcatoStaccato" => vec![A::Marcato, A::Staccato],
        "articMarcatoTenuto" => vec![A::Marcato, A::Tenuto],
        "articTenutoAccent" => vec![A::Tenuto, A::Accent],
        _ => vec![A::Other(symbol.to_string())],
    };
    articulations
        .into_iter()
        .map(ChordMark::Articulation)
        .collect()
}

/// How an [`Articulation`] is written: most as an `<Articulation>` symbol
/// in the `<Chord>`, but falls/doits/plops/scoops as a `<ChordLine>` and
/// breath marks/caesuras as a `<Breath>` after it. `None` for
/// [`Articulation::Fermata`], a `<Fermata>` of its own.
pub(crate) enum ArticulationMark {
    Symbol(String),
    ChordLine(u8),
    Breath(&'static str),
}

pub(crate) fn articulation_mark(articulation: &Articulation) -> Option<ArticulationMark> {
    let symbol = match articulation {
        Articulation::Accent => "articAccentAbove",
        Articulation::Marcato => "articMarcatoAbove",
        Articulation::Staccato => "articStaccatoAbove",
        Articulation::Tenuto => "articTenutoAbove",
        Articulation::DetachedLegato => "articTenutoStaccatoAbove",
        Articulation::Staccatissimo => "articStaccatissimoAbove",
        Articulation::Spiccato => "articStaccatissimoStrokeAbove",
        Articulation::Stress => "articStressAbove",
        Articulation::Unstress => "articUnstressAbove",
        Articulation::SoftAccent => "articSoftAccentAbove",
        // MuseScore's `ChordLineType`: NOTYPE, FALL, DOIT, PLOP, SCOOP.
        Articulation::Falloff => return Some(ArticulationMark::ChordLine(1)),
        Articulation::Doit => return Some(ArticulationMark::ChordLine(2)),
        Articulation::Plop => return Some(ArticulationMark::ChordLine(3)),
        Articulation::Scoop => return Some(ArticulationMark::ChordLine(4)),
        Articulation::BreathMark => return Some(ArticulationMark::Breath("breathMarkComma")),
        Articulation::Caesura => return Some(ArticulationMark::Breath("caesura")),
        Articulation::Fermata => return None,
        Articulation::Other(symbol) => symbol,
    };
    Some(ArticulationMark::Symbol(symbol.to_string()))
}

pub(crate) fn articulation_from_chord_line(subtype: &str) -> Option<Articulation> {
    match subtype.trim() {
        "1" => Some(Articulation::Falloff),
        "2" => Some(Articulation::Doit),
        "3" => Some(Articulation::Plop),
        "4" => Some(Articulation::Scoop),
        _ => None,
    }
}

pub(crate) fn articulation_from_breath(symbol: &str) -> Articulation {
    if symbol.contains("aesura") {
        Articulation::Caesura
    } else {
        Articulation::BreathMark
    }
}

/// The SMuFL symbol an ornament is written as (`<Ornament><subtype>`).
/// `None` for [`Ornament::Tremolo`] (a `<Tremolo>` of its own) and
/// [`Ornament::WavyLine`] (a `Trill` spanner, not written). MuseScore has
/// no delayed turns or long inverted mordent; those take the plain symbols.
pub(crate) fn ornament_symbol(ornament: &Ornament) -> Option<&str> {
    Some(match ornament {
        Ornament::Trill => "ornamentTrill",
        Ornament::Turn | Ornament::DelayedTurn => "ornamentTurn",
        Ornament::InvertedTurn | Ornament::DelayedInvertedTurn => "ornamentTurnInverted",
        Ornament::VerticalTurn => "ornamentTurnUp",
        Ornament::InvertedVerticalTurn => "ornamentTurnUpS",
        Ornament::Shake => "ornamentShake3",
        Ornament::Mordent => "ornamentMordent",
        Ornament::InvertedMordent { .. } => "ornamentShortTrill",
        Ornament::Schleifer => "ornamentPrecompSlide",
        Ornament::Haydn => "ornamentHaydn",
        Ornament::Other(symbol) => symbol,
        Ornament::Tremolo(_) | Ornament::WavyLine => return None,
    })
}

pub(crate) fn ornament_from_symbol(symbol: &str) -> Ornament {
    match symbol {
        "ornamentTrill" => Ornament::Trill,
        "ornamentTurn" => Ornament::Turn,
        "ornamentTurnInverted" => Ornament::InvertedTurn,
        "ornamentTurnUp" => Ornament::VerticalTurn,
        "ornamentTurnUpS" => Ornament::InvertedVerticalTurn,
        "ornamentShake3" | "ornamentShakeMuffat1" => Ornament::Shake,
        "ornamentMordent" => Ornament::Mordent,
        // SMuFL renamed the inverted mordent; MuseScore 3 used the old name.
        "ornamentShortTrill" | "ornamentMordentInverted" => {
            Ornament::InvertedMordent { long: false }
        }
        "ornamentPrecompSlide" => Ornament::Schleifer,
        "ornamentHaydn" => Ornament::Haydn,
        other => Ornament::Other(other.to_string()),
    }
}

/// Single-chord tremolo subtype: `r8` (one stroke) .. `r64` (four).
pub(crate) fn tremolo_subtype(strokes: u8) -> String {
    format!("r{}", 4u32 << strokes.clamp(1, 4))
}

/// Inverse of [`tremolo_subtype`]; two-chord tremolos (`c8`..) are `None`.
pub(crate) fn tremolo_from_subtype(subtype: &str) -> Option<u8> {
    match subtype.trim() {
        "r8" => Some(1),
        "r16" => Some(2),
        "r32" => Some(3),
        "r64" => Some(4),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Barlines
// ---------------------------------------------------------------------------

/// A `<BarLine>`'s style. Repeat subtypes read as regular: the repeat is
/// `<startRepeat>`/`<endRepeat>`'s.
pub(crate) fn barline_style(barline: &MscxBarLine) -> BarlineStyle {
    if !barline.visible {
        return BarlineStyle::None;
    }
    match barline.subtype.trim() {
        "dotted" => BarlineStyle::Dotted,
        "dashed" => BarlineStyle::Dashed,
        "heavy" => BarlineStyle::Heavy,
        "double" => BarlineStyle::LightLight,
        "end" => BarlineStyle::LightHeavy,
        "reverse-end" => BarlineStyle::HeavyLight,
        "double-heavy" => BarlineStyle::HeavyHeavy,
        // "tick" and "short" are normal barlines spanning part of the staff.
        _ => match barline.span_from {
            Some(from) if from < 0 => BarlineStyle::Tick,
            Some(from) if from > 0 => BarlineStyle::Short,
            _ => BarlineStyle::Regular,
        },
    }
}

/// `None` for a regular barline, which MuseScore generates rather than
/// stores.
pub(crate) fn barline_to_mscx(style: BarlineStyle) -> Option<MscxBarLine> {
    let (subtype, span, visible) = match style {
        BarlineStyle::Regular => return None,
        BarlineStyle::Dotted => ("dotted", None, true),
        BarlineStyle::Dashed => ("dashed", None, true),
        BarlineStyle::Heavy => ("heavy", None, true),
        BarlineStyle::LightLight => ("double", None, true),
        BarlineStyle::LightHeavy => ("end", None, true),
        BarlineStyle::HeavyLight => ("reverse-end", None, true),
        BarlineStyle::HeavyHeavy => ("double-heavy", None, true),
        BarlineStyle::Tick => ("normal", Some((-1, -7)), true),
        BarlineStyle::Short => ("normal", Some((2, -2)), true),
        BarlineStyle::None => ("normal", None, false),
    };
    Some(MscxBarLine {
        subtype: subtype.to_string(),
        span_from: span.map(|(from, _)| from),
        span_to: span.map(|(_, to)| to),
        visible,
    })
}

// ---------------------------------------------------------------------------
// Jumps and markers
// ---------------------------------------------------------------------------

/// A `<Jump>`/`<Marker>`'s navigation kind; `None` for one this model has
/// no kind for.
pub(crate) fn jump_kind(mark: &MscxMeasureMark) -> Option<JumpKind> {
    if is_doubled_sign(mark) {
        return None;
    }
    match mark {
        MscxMeasureMark::Marker(marker) => marker_kind(marker),
        MscxMeasureMark::Jump(jump) => jump_kind_of(jump),
    }
}

/// Whether a `<Jump>`/`<Marker>` is of the second segno or coda
/// (MuseScore's Dal Doppio Segno and al Doppia Coda), which the model has
/// no kind for: as with GP's segno segno and double coda, merging them
/// into the segno and coda would send playback to the wrong sign.
pub(crate) fn is_doubled_sign(mark: &MscxMeasureMark) -> bool {
    match mark {
        MscxMeasureMark::Marker(marker) => {
            DOUBLED_LABELS.contains(&marker.label.trim())
                || doubled_text(&marker.text.trim().to_ascii_lowercase())
        }
        MscxMeasureMark::Jump(jump) => {
            DOUBLED_LABELS.contains(&jump.jump_to.trim())
                || DOUBLED_LABELS.contains(&jump.play_until.trim())
                || doubled_text(&jump.text.trim().to_ascii_lowercase())
        }
    }
}

/// The GP direction sign of a second segno or coda mark
/// ([`is_doubled_sign`]), which the model keeps for a GP export (and so
/// playback can tell it skips the jump): `None` for any other mark.
pub(crate) fn doubled_direction(mark: &MscxMeasureMark) -> Option<DirectionSign> {
    if !is_doubled_sign(mark) {
        return None;
    }
    Some(match mark {
        MscxMeasureMark::Marker(marker) => match marker.label.trim() {
            "varsegno" => DirectionSign::SegnoSegno,
            "varcoda" => DirectionSign::DaDoubleCoda,
            "codetta" => DirectionSign::DoubleCoda,
            _ if marker.text.to_ascii_lowercase().contains("segno") => DirectionSign::SegnoSegno,
            _ => DirectionSign::DoubleCoda,
        },
        MscxMeasureMark::Jump(jump) => {
            let text = jump.text.trim().to_ascii_lowercase();
            let jump_to = jump.jump_to.trim();
            let until = match jump.play_until.trim() {
                "varcoda" | "codetta" => "double coda",
                "" if text.contains("doppia coda") => "double coda",
                "" if text.ends_with("fine") => "fine",
                "" if text.ends_with("coda") => "coda",
                "fine" => "fine",
                "coda" | "tocoda" => "coda",
                _ => "",
            };
            let segno_segno = jump_to == "varsegno"
                || (jump_to != "start"
                    && jump_to != "segno"
                    && ["d.s.s.", "d.d.s.", "dal doppio segno"]
                        .iter()
                        .any(|prefix| text.starts_with(prefix)));
            let da_capo = !segno_segno
                && (jump_to == "start"
                    || (jump_to != "segno"
                        && (text.starts_with("d.c.") || text.starts_with("da capo"))));
            match (segno_segno, da_capo, until) {
                (true, _, "fine") => DirectionSign::DaSegnoSegnoAlFine,
                (true, _, "coda") => DirectionSign::DaSegnoSegnoAlCoda,
                (true, _, "double coda") => DirectionSign::DaSegnoSegnoAlDoubleCoda,
                (true, _, _) => DirectionSign::DaSegnoSegno,
                (false, true, _) => DirectionSign::DaCapoAlDoubleCoda,
                (false, false, _) => DirectionSign::DaSegnoAlDoubleCoda,
            }
        }
    })
}

/// MuseScore's labels of the second segno ("varsegno"), the sign an al
/// Doppia Coda pass leaves at ("varcoda") and the doppia coda
/// ("codetta").
const DOUBLED_LABELS: [&str; 3] = ["varsegno", "varcoda", "codetta"];

/// Whether a (lowercase) text names the second segno or coda.
fn doubled_text(text: &str) -> bool {
    ["d.s.s.", "d.d.s.", "dal doppio segno"]
        .iter()
        .any(|prefix| text.starts_with(prefix))
        || text.contains("doppia coda")
}

/// A `<Marker>` by its `<label>`, else its text.
fn marker_kind(marker: &MscxMarker) -> Option<JumpKind> {
    match marker.label.trim() {
        "segno" => Some(JumpKind::Segno),
        // MuseScore's labels are historical: the coda sign is "codab" and
        // "To Coda" is "coda".
        "codab" => Some(JumpKind::Coda),
        "fine" => Some(JumpKind::Fine),
        "coda" | "tocoda" | "tocodasym" => Some(JumpKind::ToCoda),
        _ => match marker.text.trim().to_ascii_lowercase().as_str() {
            "fine" => Some(JumpKind::Fine),
            "to coda" => Some(JumpKind::ToCoda),
            "segno" => Some(JumpKind::Segno),
            "coda" => Some(JumpKind::Coda),
            _ => None,
        },
    }
}

/// A `<Jump>` by where it jumps to (`start`: D.C., a segno: D.S.) and
/// where playback stops (`fine`/`coda`), else by its text. One to any
/// other label (a coda, a user's marker) has no kind unless its text
/// names a D.C. or D.S.
fn jump_kind_of(jump: &MscxJump) -> Option<JumpKind> {
    let text = jump.text.trim().to_ascii_lowercase();
    let da_capo = match jump.jump_to.trim() {
        "start" => true,
        "segno" => false,
        _ => {
            if text.starts_with("d.c.") || text.starts_with("da capo") {
                true
            } else if text.starts_with("d.s.") || text.starts_with("dal segno") {
                false
            } else {
                return None;
            }
        }
    };
    let until = match jump.play_until.trim() {
        "" if text.ends_with("fine") => "fine",
        "" if text.ends_with("coda") => "coda",
        other => other,
    };
    Some(match (da_capo, until) {
        (true, "fine") => JumpKind::DaCapoAlFine,
        (true, "coda" | "tocoda") => JumpKind::DaCapoAlCoda,
        (true, _) => JumpKind::DaCapo,
        (false, "fine") => JumpKind::DalSegnoAlFine,
        (false, "coda" | "tocoda") => JumpKind::DalSegnoAlCoda,
        (false, _) => JumpKind::DalSegno,
    })
}

/// The `<Jump>`/`<Marker>` for a navigation kind; `None` for repeats,
/// which are `<startRepeat>`/`<endRepeat>`. Marker texts are MSCX markup.
pub(crate) fn jump_kind_to_mscx(kind: JumpKind) -> Option<MscxMeasureMark> {
    let marker = |label: &str, text: &str| {
        Some(MscxMeasureMark::Marker(MscxMarker {
            label: label.to_string(),
            text: text.to_string(),
        }))
    };
    let jump = |text: &str, jump_to: &str, play_until: &str, continue_at: &str| {
        Some(MscxMeasureMark::Jump(MscxJump {
            text: text.to_string(),
            jump_to: jump_to.to_string(),
            play_until: play_until.to_string(),
            continue_at: continue_at.to_string(),
        }))
    };
    match kind {
        JumpKind::RepeatOpen | JumpKind::RepeatClose => None,
        JumpKind::Segno => marker("segno", "<sym>segno</sym>"),
        JumpKind::Coda => marker("codab", "<sym>coda</sym>"),
        JumpKind::Fine => marker("fine", "Fine"),
        JumpKind::ToCoda => marker("coda", "To Coda"),
        JumpKind::DaCapo => jump("D.C.", "start", "end", ""),
        JumpKind::DaCapoAlFine => jump("D.C. al Fine", "start", "fine", ""),
        JumpKind::DaCapoAlCoda => jump("D.C. al Coda", "start", "coda", "codab"),
        JumpKind::DalSegno => jump("D.S.", "segno", "end", ""),
        JumpKind::DalSegnoAlFine => jump("D.S. al Fine", "segno", "fine", ""),
        JumpKind::DalSegnoAlCoda => jump("D.S. al Coda", "segno", "coda", "codab"),
    }
}
