use std::ops::Add;

use fraction::ToPrimitive;

use crate::io::*;

/// Type of the chord.
#[derive(Clone,PartialEq)]
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
#[derive(Clone,PartialEq)]
pub enum ChordAlteration {
    /// Perfect.
    Perfect,
    /// Diminished.
    Diminished,
    /// Augmented.
    Augmented,
}

/// Extension type of the chord
#[derive(Clone,PartialEq)]
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
#[derive(Clone,PartialEq)]
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
#[derive(Clone,PartialEq)]
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
    pub strings: Vec<i8>,
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
    /// Read chord diagram. First byte is chord header. If it's set to 0, then following chord is written in 
    /// default (GP3) format. If chord header is set to 1, then chord diagram in encoded in more advanced (GP4) format.
    pub fn read(data: &Vec<u8>, seek: &mut usize, string_count: u8) -> Chord {
        let mut c = Chord {length: string_count, ..Default::default()};
        for _ in 0..string_count {c.strings.push(-1);}
        c.new_format = Some(read_bool(data, seek));
        if c.new_format == Some(true) {c.read_new(data, seek);}
        else {c.read_new(data, seek);}
        return c;
    }
    /// Read chord diagram encoded in GP3 format. Chord diagram is read as follows:
    /// - Name: `int-byte-size-string`. Name of the chord, e.g. *Em*.
    /// - First fret: `int`. The fret from which the chord is displayed in chord editor.
    /// - List of frets: 6 `ints`. Frets are listed in order: fret on the string 1, fret on the string 2, ..., fret on the
    /// string 6. If string is untouched then the values of fret is *-1*.
    fn read_old(&mut self, data: &Vec<u8>, seek: &mut usize) {
        self.name = read_int_size_string(data, seek);
        self.first_fret = Some(read_int(data, seek).to_u8().unwrap());
        if self.first_fret.is_some() {
            for i in 0u8..6u8 {
                let fret = read_int(data, seek).to_i8().unwrap();
                if i < self.strings.len().to_u8().unwrap() {self.strings.push(fret);} //self.strings[i] = fret;
            }
        }
    }
    /// Read new-style (GP4) chord diagram. New-style chord diagram is read as follows:
    /// - Sharp: `bool`. If true, display all semitones as sharps, otherwise display as flats.
    /// - Blank space, 3 `Bytes <byte>`.
    /// - Root: `int`. Values are:
    ///   * -1 for customized chords
    ///   *  0: C
    ///   *  1: C#
    ///   * ...
    /// - Type: `int`. Determines the chord type as followed. See `ChordType` for mapping.
    /// - Chord extension: `int`. See `ChordExtension` for mapping.
    /// - Bass note: `int`. Lowest note of chord as in *C/Am*.
    /// - Tonality: `int`. See `ChordAlteration` for mapping.
    /// - Add: `bool`. Determines if an "add" (added note) is present in the chord.
    /// - Name: `byte-size-string`. Max length is 22.
    /// - Fifth alteration: `int`. Maps to `ChordAlteration`.
    /// - Ninth alteration: `int`. Maps to `ChordAlteration`.
    /// - Eleventh alteration: `int`. Maps to `ChordAlteration`.
    /// - List of frets: 6 `Ints <int>`. Fret values are saved as in default format.
    /// - Count of barres: `int`. Maximum count is 2.
    /// - Barre frets: 2 `Ints <int>`.
    /// - Barre start strings: 2 `Ints <int>`.
    /// - Barre end string: 2 `Ints <int>`.
    /// - Omissions: 7 `Bools <bool>`. If the value is true then note is played in chord.
    /// - Blank space, 1 `byte`.
    fn read_new(&mut self, data: &Vec<u8>, seek: &mut usize) {
        self.sharp = Some(read_bool(data, seek));
        *seek += 3;
        self.root = Some(PitchClass::from(read_int(data, seek).to_i8().unwrap(), None, self.sharp));
        self.kind = Some(match read_int(data, seek) {
            0  => ChordType::Major,
            1  => ChordType::Seventh,
            2  => ChordType::MajorSeventh,
            3  => ChordType::Sixth,
            4  => ChordType::Minor,
            5  => ChordType::MinorSeventh,
            6  => ChordType::MinorMajor,
            7  => ChordType::MinorSixth,
            8  => ChordType::SuspendedSecond,
            9  => ChordType::SuspendedFourth,
            10 => ChordType::SeventhSuspendedSecond,
            11 => ChordType::SeventhSuspendedFourth,
            12 => ChordType::Diminished,
            13 => ChordType::Augmented,
            14 => ChordType::Power,
            _  => panic!("Cannot read chord type (new format)"),
        });
        self.extension = Some(match read_int(data, seek) {
            0 => ChordExtension::None,
            1 => ChordExtension::Ninth,
            2 => ChordExtension::Eleventh,
            3 => ChordExtension::Thirteenth,
            _ => panic!("Cannot read chord type (new format)"),
        });
        self.bass = Some(PitchClass::from(read_int(data, seek).to_i8().unwrap(), None, self.sharp));
        self.tonality = Some(match read_int(data, seek) {
            0 => ChordAlteration::Perfect,
            1 => ChordAlteration::Diminished,
            2 => ChordAlteration::Augmented,
            _ => panic!("Cannot read chord fifth (new format)"),
        });
        self.add = Some(read_bool(data, seek));
        self.name = read_byte_size_string(data, seek);
        *seek += 22 - self.name.len();
        self.fifth = Some(match read_int(data, seek) {
            0 => ChordAlteration::Perfect,
            1 => ChordAlteration::Diminished,
            2 => ChordAlteration::Augmented,
            _ => panic!("Cannot read chord fifth (new format)"),
        });
        self.ninth = Some(match read_int(data, seek) {
            0 => ChordAlteration::Perfect,
            1 => ChordAlteration::Diminished,
            2 => ChordAlteration::Augmented,
            _ => panic!("Cannot read chord fifth (new format)"),
        });
        self.eleventh = Some(match read_int(data, seek) {
            0 => ChordAlteration::Perfect,
            1 => ChordAlteration::Diminished,
            2 => ChordAlteration::Augmented,
            _ => panic!("Cannot read chord fifth (new format)"),
        });
        self.first_fret = Some(read_int(data, seek).to_u8().unwrap());
        for i in 0u8..6u8 {
            let fret = read_int(data, seek).to_i8().unwrap();
            if i < self.strings.len().to_u8().unwrap() {self.strings.push(fret);} //self.strings[i] = fret;
        }
        //barre
        let barre_count = read_int(data, seek).to_usize().unwrap();
        let mut barre_frets:  Vec<i32> = Vec::with_capacity(2);
        let mut barre_starts: Vec<i32> = Vec::with_capacity(2);
        let mut barre_ends:   Vec<i32> = Vec::with_capacity(2);
        for _ in 0u8..2u8 {barre_frets.push(read_int(data, seek));}
        for _ in 0u8..2u8 {barre_starts.push(read_int(data, seek));}
        for _ in 0u8..2u8 {barre_ends.push(read_int(data, seek));}
        for i in 0..barre_count {self.barres.push(Barre{fret:barre_frets[i].to_i8().unwrap(), start:barre_starts[i].to_i8().unwrap(), end:barre_ends[i].to_i8().unwrap()});}

        for _ in 0u8..7u8 {self.omissions.push(read_bool(data, seek));}
        *seek += 1;
    }
}

/// A single barre
#[derive(Clone,PartialEq)]
pub struct Barre {
    pub fret: i8,
    /// First string from the bottom of the barre
    pub start: i8,
    /// ast string on the top of the barre
    pub end: i8,
}


pub const SHARP_NOTES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
pub const FLAT_NOTES:  [&str; 12] = ["C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B"];

#[derive(Clone, PartialEq)]
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
