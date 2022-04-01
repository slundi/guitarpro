

/// Type of the chord.
#[derive(Clone)]
pub enum ChordType {
    /// Major chord.
    Major,
    /// Dominant seventh chord.
    Seventh,
    /// Major seventh chord.
    MajorSeventh,
    /// Add sixth chord.
    Sixth,
    /// Minor chord.
    Minor,
    /// Minor seventh chord.
    MinorSeventh,
    /// Minor major seventh chord.
    MinorMajor,
    /// Minor add sixth chord.
    MinorSixth,
    /// Suspended second chord.
    SuspendedSecond,
    /// Suspended fourth chord.
    SuspendedFourth,
    /// Seventh suspended second chord.
    SeventhSuspendedSecond,
    /// Seventh suspended fourth chord.
    SeventhSuspendedFourth,
    /// Diminished chord.
    Diminished,
    /// Augmented chord.
    Augmented,
    /// Power chord.
    Power,
}

/// Tonality of the chord
#[derive(Clone)]
pub enum ChordAlteration {
    /// Perfect.
    Perfect,
    /// Diminished.
    Diminished,
    /// Augmented.
    Augmented,
}

/// Extension type of the chord
#[derive(Clone)]
pub enum ChordExtension {
    None,
    /// Ninth chord.
    Ninth,
    /// Eleventh chord.
    Eleventh,
    /// Thirteenth chord.
    Thirteenth
}

//TODO: move fingering to note?
/// Left and right hand fingering used in tabs and chord diagram editor.
#[derive(Clone)]
pub enum Fingering {
    /// Open or muted.
    Open, //-1?
    /// Thumb.
    Thumb,
    /// Index finger.
    Index,
    /// Middle finger.
    Middle,
    /// Annular finger.
    Annular,
    /// Little finger.
    Little,
}
/// A chord annotation for beats
#[derive(Clone)]
pub struct Chord {
    pub length: u8,
    pub sharp: Option<bool>,
    pub root: Option<PitchClass>,
    pub kind: Option<ChordType>,
    pub extension: Option<ChordExtension>,
    pub bass: Option<PitchClass>,
    pub tonality: Option<ChordAlteration>,
    pub add: Option<bool>,
    pub name: String,
    pub fifth: Option<ChordAlteration>,
    pub ninth: Option<ChordAlteration>,
    pub eleventh: Option<ChordAlteration>,
    pub first_fret: Option<u8>,
    pub strings: Vec<u8>,
    pub barres: Vec<Barre>,
    pub omissions: Vec<bool>,
    pub fingerings: Vec<Fingering>,
    pub show: Option<bool>,
    pub new_format: Option<bool>,

    //TODO: def __attrs_post_init__(self): self.strings = <-1>, * self.length
    //TODO: @property def notes(self): return <string for string in self.strings if string >= 0>,
}
impl Default for Chord {
    fn default() -> Self { Chord {
        length: 0,
        sharp:None, root:None, kind:None, extension:None, bass:None, tonality:None, add:None, name:String::new(),
        fifth:None, ninth:None, eleventh:None,
        first_fret: None, strings:Vec::new(), barres:Vec::new(), omissions:Vec::new(), fingerings:Vec::new(),
        show:None, new_format:None,
    }}
}
impl Chord {
    //TODO: @property def notes(self): return [string for string in self.strings if string >= 0]
}

/// A single barre
#[derive(Clone)]
pub struct Barre {
    pub fret: u8,
    /// First string from the bottom of the barre
    pub start: u8,
    /// ast string on the top of the barre
    pub end: u8,
}


pub const SHARP_NOTES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
pub const FLAT_NOTES:  [&str; 12] = ["C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B"];

#[derive(Clone)]
pub struct PitchClass {
    pub note: String,
    pub just: i8,
    /// flat (-1), none (0) or sharp (1).
    pub accidental: i8,
    pub value: i8,
    pub sharp: bool,
}
impl PitchClass {

    pub fn from(just: i8, accidental: Option<i8>, sharp: Option<bool>) -> PitchClass {
        let mut p = PitchClass {just:just, accidental:0, value:-1, sharp: true, note:String::with_capacity(2) };
        p.value = p.just % 12;
        println!("VALUE: {}", p.value);
        let mut pitch = 0i8;
        if accidental.is_none() {
            p.note=String::from(SHARP_NOTES[p.value as usize]); //try: note = SHARP_NOTES[p.value]; except KeyError: note = FLAT_NOTES[p.value];
            //if FLAT_NOTES[p.value]  == &note {note=String::from(FLAT_NOTES[p.value]);  p.sharp = false;} 
            if p.note.ends_with("b")      {p.accidental = -1; p.sharp = false;}
            else if p.note.ends_with("#") {p.accidental = 1;}
            pitch = p.value - p.accidental;
        } else {
            pitch = p.value;
            p.accidental = accidental.unwrap();
        }
        println!("NOTE: {}", p.note);
        p.just = pitch % 12;
        p.value = p.just + p.accidental;
        if sharp.is_none() { p.sharp = p.accidental >= 0; }
        return p;
    }
    pub fn from_note(note: String) -> PitchClass {
        let mut p = PitchClass {note:note, just:0, accidental:0, value:-1, sharp: true,};
        if p.note.ends_with("b")      {p.accidental = -1; p.sharp = false;}
        else if p.note.ends_with("#") {p.accidental = 1;}
        for i in 0i8..12i8 {
            if SHARP_NOTES[i as usize] == &p.note || FLAT_NOTES[i as usize] == &p.note {p.value = i; break;}
        }
        let pitch = p.value - p.accidental; 
        p.just = pitch % 12;
        p.value = p.just + p.accidental;
        return p;
    }
    pub fn to_string(&self) -> String {
        if self.sharp {return String::from(SHARP_NOTES[self.value as usize]);}
        else          {return String::from(FLAT_NOTES[self.value as usize]);}
    }
}

#[cfg(test)]
mod test {
    use crate::chord::PitchClass;

    #[test]
    fn test_pitches() {
        // 1
        let p = PitchClass::from_note("D#".to_string());
        assert_eq!(true, p.sharp, "D# is sharp? {}", true);
        assert_eq!(1, p.accidental);
        //2
        let p = PitchClass::from(4, Some(-1), None);
        assert_eq!(3, p.value);
        assert_eq!(false, p.sharp);
        assert_eq!("Eb", p.to_string(), "Note should be Eb");
        //3
        let p = PitchClass::from(4, Some(-1), Some(true));
        assert_eq!(3, p.value);
        assert_eq!("D#", p.to_string(), "Note should be D#");
        //4
        let p = PitchClass::from(3, None, None);
        assert_eq!("Eb", p.to_string(), "Note should be Eb");
        //5
        let p = PitchClass::from(3, None, Some(true));
        assert_eq!("D#", p.to_string(), "Note should be D#");
    }
}
