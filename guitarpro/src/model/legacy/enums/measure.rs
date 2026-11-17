/// An enumeration of different triplet feels.
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TripletFeel {
    None,
    Eighth,
    Sixteenth,
}
// Some GP5 exports write non-canonical bytes here; Guitar Pro itself falls back
// to `None`, so we accept any value instead of erroring.
pub(crate) fn get_triplet_feel(value: u8) -> TripletFeel {
    match value {
        1 => TripletFeel::Eighth,
        2 => TripletFeel::Sixteenth,
        _ => TripletFeel::None,
    }
}
pub(crate) fn from_triplet_feel(value: &TripletFeel) -> u8 {
    match value {
        TripletFeel::None => 0,
        TripletFeel::Eighth => 1,
        TripletFeel::Sixteenth => 2,
    }
}

/// An enumeration of available clefs
#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone)]
pub enum MeasureClef {
    Treble,
    Bass,
    Tenor,
    Alto,
}

/// A line break directive: `NONE: no line break`, `BREAK: break line`, `Protect the line from breaking`.
#[repr(u8)]
#[derive(Debug, Clone)]
pub enum LineBreak {
    None,
    Break,
    Protect,
}
pub(crate) fn get_line_break(value: u8) -> LineBreak {
    match value {
        1 => LineBreak::Break,
        2 => LineBreak::Protect,
        _ => LineBreak::None,
    }
}
pub(crate) fn from_line_break(value: &LineBreak) -> u8 {
    match value {
        LineBreak::None => 0,
        LineBreak::Break => 1,
        LineBreak::Protect => 2,
    }
}

/// A navigation sign like *Coda* (𝄌: U+1D10C) or *Segno* (𝄋 or 𝄉: U+1D10B or U+1D109).
/// Declared (and ordered) as GP5 stores them.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DirectionSign {
    Coda,
    DoubleCoda,
    Segno,
    SegnoSegno,
    Fine,
    DaCapo,
    DaCapoAlCoda,
    DaCapoAlDoubleCoda,
    DaCapoAlFine,
    DaSegno,
    DaSegnoAlCoda,
    DaSegnoAlDoubleCoda,
    DaSegnoAlFine,
    DaSegnoSegno,
    DaSegnoSegnoAlCoda,
    DaSegnoSegnoAlDoubleCoda,
    DaSegnoSegnoAlFine,
    DaCoda,
    DaDoubleCoda,
}

/// Every sign with its GPIF name and its label, in declaration order (as
/// GP5 stores them): the one table [`DirectionSign`]'s lookups read.
const SIGNS: [(DirectionSign, &str, &str); 19] = [
    (DirectionSign::Coda, "Coda", "Coda"),
    (DirectionSign::DoubleCoda, "DoubleCoda", "Double Coda"),
    (DirectionSign::Segno, "Segno", "Segno"),
    (DirectionSign::SegnoSegno, "SegnoSegno", "Segno Segno"),
    (DirectionSign::Fine, "Fine", "Fine"),
    (DirectionSign::DaCapo, "DaCapo", "D.C."),
    (DirectionSign::DaCapoAlCoda, "DaCapoAlCoda", "D.C. al Coda"),
    (
        DirectionSign::DaCapoAlDoubleCoda,
        "DaCapoAlDoubleCoda",
        "D.C. al Double Coda",
    ),
    (DirectionSign::DaCapoAlFine, "DaCapoAlFine", "D.C. al Fine"),
    (DirectionSign::DaSegno, "DaSegno", "D.S."),
    (
        DirectionSign::DaSegnoAlCoda,
        "DaSegnoAlCoda",
        "D.S. al Coda",
    ),
    (
        DirectionSign::DaSegnoAlDoubleCoda,
        "DaSegnoAlDoubleCoda",
        "D.S. al Double Coda",
    ),
    (
        DirectionSign::DaSegnoAlFine,
        "DaSegnoAlFine",
        "D.S. al Fine",
    ),
    (DirectionSign::DaSegnoSegno, "DaSegnoSegno", "D.S.S."),
    (
        DirectionSign::DaSegnoSegnoAlCoda,
        "DaSegnoSegnoAlCoda",
        "D.S.S. al Coda",
    ),
    (
        DirectionSign::DaSegnoSegnoAlDoubleCoda,
        "DaSegnoSegnoAlDoubleCoda",
        "D.S.S. al Double Coda",
    ),
    (
        DirectionSign::DaSegnoSegnoAlFine,
        "DaSegnoSegnoAlFine",
        "D.S.S. al Fine",
    ),
    (DirectionSign::DaCoda, "DaCoda", "Da Coda"),
    (
        DirectionSign::DaDoubleCoda,
        "DaDoubleCoda",
        "Da Double Coda",
    ),
];

// Each sign sits at its own discriminant, and none is missing.
const _: () = {
    assert!(DirectionSign::DaDoubleCoda as usize + 1 == SIGNS.len());
    let mut index = 0;
    while index < SIGNS.len() {
        assert!(SIGNS[index].0 as usize == index);
        index += 1;
    }
};

impl DirectionSign {
    /// Every sign, as GP5 stores them.
    pub const ALL: [DirectionSign; 19] = {
        let mut all = [DirectionSign::Coda; 19];
        let mut index = 0;
        while index < SIGNS.len() {
            all[index] = SIGNS[index].0;
            index += 1;
        }
        all
    };

    /// The sign's name, as GPIF writes it.
    pub fn name(self) -> &'static str {
        SIGNS[self as usize].1
    }

    /// The sign as a reader knows it (D.C., D.S. al Coda, …).
    pub fn label(self) -> &'static str {
        SIGNS[self as usize].2
    }

    /// Inverse of [`DirectionSign::name`].
    pub fn from_name(name: &str) -> Option<Self> {
        SIGNS
            .iter()
            .find(|(_, sign_name, _)| *sign_name == name)
            .map(|(sign, _, _)| *sign)
    }

    /// Whether it marks a place to jump to (a GPIF `<Target>`), not a jump
    /// (a `<Jump>`).
    pub fn is_target(self) -> bool {
        matches!(
            self,
            DirectionSign::Coda
                | DirectionSign::DoubleCoda
                | DirectionSign::Segno
                | DirectionSign::SegnoSegno
                | DirectionSign::Fine
        )
    }
}
