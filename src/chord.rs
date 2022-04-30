use fraction::ToPrimitive;

use crate::{io::*, gp::*, enums::*};

/// A chord annotation for beats
#[derive(Debug,Clone,PartialEq,Default)]
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

    //TODO: @property def notes(self): return <string for string in self.strings if string >= 0>,
}
/*impl Default for Chord {
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
}*/


/// A single barre
#[derive(Debug,Clone,PartialEq)]
pub struct Barre {
    pub fret: i8,
    /// First string from the bottom of the barre
    pub start: i8,
    /// ast string on the top of the barre
    pub end: i8,
}


pub const SHARP_NOTES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
pub const FLAT_NOTES:  [&str; 12] = ["C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B"];

#[derive(Debug,Clone, PartialEq)]
pub struct PitchClass {
    pub note: String,
    pub just: i8,
    /// flat (-1), none (0) or sharp (1).
    pub accidental: i8,
    pub value: i8,
    pub sharp: bool,
}
impl PitchClass {
    pub(crate) fn from(just: i8, accidental: Option<i8>, sharp: Option<bool>) -> PitchClass {
        let mut p = PitchClass {just, accidental:0, value:-1, sharp: true, note:String::with_capacity(2) };
        let pitch: i8;
        let accidental2: i8;
        if accidental == None {
            let value = p.just % 12;
            //println!("PitchClass(), value: {}", value);
            p.note = if value >= 0 {String::from(SHARP_NOTES[value as usize])} else {String::from(SHARP_NOTES[(12 + value).to_usize().unwrap()])}; //try: note = SHARP_NOTES[p.value]; except KeyError: note = FLAT_NOTES[p.value];
            //if FLAT_NOTES[p.value]  == &note {note=String::from(FLAT_NOTES[p.value]);  p.sharp = false;} 
            if      p.note.ends_with('b') {accidental2 = -1; p.sharp = false;}
            else if p.note.ends_with('#') {accidental2 = 1;}
            else                          {accidental2 = 0;}
            pitch = value - accidental2;
        } else {
            pitch = p.just;
            accidental2 = accidental.unwrap();
        }
        //println!("VALUE: {} \t NOTE: {}", p.value, p.note);
        p.just = pitch % 12;
        p.accidental = accidental2;
        p.value = p.just + accidental2;
        if sharp.is_none() { p.sharp = p.accidental >= 0; }
        p
    }
    pub(crate) fn from_note(note: String) -> PitchClass {
        let mut p = PitchClass {note, just:0, accidental:0, value:-1, sharp: true,};
        if p.note.ends_with('b')      {p.accidental = -1; p.sharp = false;}
        else if p.note.ends_with('#') {p.accidental = 1;}
        for i in 0i8..12i8 {
            if SHARP_NOTES[i as usize] == p.note || FLAT_NOTES[i as usize] == p.note {p.value = i; break;}
        }
        let pitch = p.value - p.accidental; 
        p.just = pitch % 12;
        p.value = p.just + p.accidental;
        p
    }
}

impl std::fmt::Display for PitchClass {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.sharp { write!(f, "{}", SHARP_NOTES[self.value as usize]) }
        else          { write!(f, "{}", FLAT_NOTES[self.value as usize]) }
    }
}

impl Song {
    /// Read chord diagram. First byte is chord header. If it's set to 0, then following chord is written in 
    /// default (GP3) format. If chord header is set to 1, then chord diagram in encoded in more advanced (GP4) format.
    pub(crate) fn read_chord(&self, data: &[u8], seek: &mut usize, string_count: u8) -> Chord {
        let mut c = Chord {length: string_count, strings: vec![-1; string_count.into()], ..Default::default()};
        for _ in 0..string_count {c.strings.push(-1);}
        c.new_format = Some(read_bool(data, seek));
        if c.new_format == Some(true) {
            if      self.version.number.0 == 3 { self.read_new_format_chord_v3(data, seek, &mut c); }
            else                               { self.read_new_format_chord_v4(data, seek, &mut c);}
        }
        else {self.read_old_format_chord(data, seek, &mut c);}
        c
    }
    /// Read chord diagram encoded in GP3 format. Chord diagram is read as follows:
    /// - Name: `int-byte-size-string`. Name of the chord, e.g. *Em*.
    /// - First fret: `int`. The fret from which the chord is displayed in chord editor.
    /// - List of frets: 6 `ints`. Frets are listed in order: fret on the string 1, fret on the string 2, ..., fret on the
    /// string 6. If string is untouched then the values of fret is *-1*.
    fn read_old_format_chord(&self, data: &[u8], seek: &mut usize, chord: &mut Chord) {
        chord.name = read_int_size_string(data, seek);
        chord.first_fret = Some(read_int(data, seek).to_u8().unwrap());
        if chord.first_fret.is_some() {
            for i in 0u8..6u8 {
                let fret = read_int(data, seek).to_i8().unwrap();
                if i < chord.strings.len().to_u8().unwrap() {chord.strings.push(fret);} //chord.strings[i] = fret;
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
    fn read_new_format_chord_v3(&self, data: &[u8], seek: &mut usize, chord: &mut Chord) {
        chord.sharp = Some(read_bool(data, seek));
        *seek += 3;
        chord.root = Some(PitchClass::from(read_int(data, seek).to_i8().unwrap(), None, chord.sharp));
        chord.kind = Some(get_chord_type(read_int(data, seek).to_u8().unwrap()));
        chord.extension = Some(get_chord_extension(read_int(data, seek).to_u8().unwrap()));
        chord.bass = Some(PitchClass::from(read_int(data, seek).to_i8().unwrap(), None, chord.sharp));
        chord.tonality = Some(get_chord_alteration(read_int(data, seek).to_u8().unwrap()));
        chord.add = Some(read_bool(data, seek));
        chord.name = read_byte_size_string(data, seek, 22);
        chord.fifth = Some(get_chord_alteration(read_int(data, seek).to_u8().unwrap()));
        chord.ninth = Some(get_chord_alteration(read_int(data, seek).to_u8().unwrap()));
        chord.eleventh = Some(get_chord_alteration(read_int(data, seek).to_u8().unwrap()));
        chord.first_fret = Some(read_int(data, seek).to_u8().unwrap());
        for i in 0u8..6u8 {
            let fret = read_int(data, seek).to_i8().unwrap();
            if i < chord.strings.len().to_u8().unwrap() {chord.strings.push(fret);} //chord.strings[i] = fret;
        }
        //barre
        let barre_count = read_int(data, seek).to_usize().unwrap();
        let mut barre_frets:  Vec<i32> = Vec::with_capacity(2);
        let mut barre_starts: Vec<i32> = Vec::with_capacity(2);
        let mut barre_ends:   Vec<i32> = Vec::with_capacity(2);
        for _ in 0u8..2u8 {barre_frets.push(read_int(data, seek));}
        for _ in 0u8..2u8 {barre_starts.push(read_int(data, seek));}
        for _ in 0u8..2u8 {barre_ends.push(read_int(data, seek));}
        for i in 0..barre_count {chord.barres.push(Barre{fret:barre_frets[i].to_i8().unwrap(), start:barre_starts[i].to_i8().unwrap(), end:barre_ends[i].to_i8().unwrap()});}

        for _ in 0u8..7u8 {chord.omissions.push(read_bool(data, seek));}
        *seek += 1;
    }

    /// Read new-style (GP4) chord diagram. New-style chord diagram is read as follows:
    /// - Sharp: `bool`. If true, display all semitones as sharps, otherwise display as flats.
    /// - Blank space, 3 `Bytes <byte>`.
    /// - Root: `byte`. Values are:
    ///   * -1 for customized chords
    ///   *  0: C
    ///   *  1: C#
    ///   * ...
    /// - Type: `byte`. Determines the chord type as followed. See `ChordType` for mapping.
    /// - Chord extension: `byte`. See `ChordExtension` for mapping.
    /// - Bass note: `int`. Lowest note of chord as in *C/Am*.
    /// - Tonality: `int`. See `ChordAlteration` for mapping.
    /// - Add: `bool`. Determines if an "add" (added note) is present in the chord.
    /// - Name: `byte-size-string`. Max length is 22.
    /// - Fifth tonality: `byte`. Maps to `ChordExtension`.
    /// - Ninth tonality: `byte`. Maps to `ChordExtension`.
    /// - Eleventh tonality: `byte`. Maps to `ChordExtension`.
    /// - List of frets: 6 `Ints <int>`. Fret values are saved as in default format.
    /// - Count of barres: `byte`. Maximum count is 5.
    /// - Barre frets: 5 `Bytes <byte>`.
    /// - Barre start strings: 5 `Bytes <byte>`.
    /// - Barre end string: 5 `Bytes <byte>`.
    /// - Omissions: 7 `Bools <bool>`. If the value is true then note is played in chord.
    /// - Blank space, 1 `byte`.
    /// - Fingering: 7 `SignedBytes <signed-byte>`. For value mapping, see `Fingering`.
    fn read_new_format_chord_v4(&self, data: &[u8], seek: &mut usize, chord: &mut Chord) {
        chord.sharp = Some(read_bool(data, seek));
        *seek += 3;
        chord.root = Some(PitchClass::from(read_byte(data, seek).to_i8().unwrap(), None, chord.sharp));
        chord.kind = Some(get_chord_type(read_byte(data, seek)));
        chord.extension = Some(get_chord_extension(read_byte(data, seek)));
        let i = read_int(data, seek);
        //println!("{:?}", i);
        chord.bass = Some(PitchClass::from(i.to_i8().unwrap(), None, chord.sharp));
        chord.tonality = Some(get_chord_alteration(read_int(data, seek).to_u8().unwrap()));
        chord.add = Some(read_bool(data, seek));
        chord.name = read_byte_size_string(data, seek, 22);
        chord.fifth = Some(get_chord_alteration(read_byte(data, seek)));
        chord.ninth = Some(get_chord_alteration(read_byte(data, seek)));
        chord.eleventh = Some(get_chord_alteration(read_byte(data, seek)));
        chord.first_fret = Some(read_int(data, seek).to_u8().unwrap());
        for i in 0u8..7u8 {
            let fret = read_int(data, seek).to_i8().unwrap();
            if i < chord.strings.len().to_u8().unwrap() {chord.strings.push(fret);} //chord.strings[i] = fret;
        }
        //barre
        let barre_count = read_byte(data, seek).to_usize().unwrap();
        let mut barre_frets:  Vec<u8> = Vec::with_capacity(5);
        let mut barre_starts: Vec<u8> = Vec::with_capacity(5);
        let mut barre_ends:   Vec<u8> = Vec::with_capacity(5);
        for _ in 0u8..5u8 {barre_frets.push(read_byte(data, seek));}
        for _ in 0u8..5u8 {barre_starts.push(read_byte(data, seek));}
        for _ in 0u8..5u8 {barre_ends.push(read_byte(data, seek));}
        for i in 0..barre_count {chord.barres.push(Barre{fret:barre_frets[i].to_i8().unwrap(), start:barre_starts[i].to_i8().unwrap(), end:barre_ends[i].to_i8().unwrap()});}
        for _ in 0u8..7u8 {chord.omissions.push(read_bool(data, seek));}
        *seek += 1;
        for _ in 0u8..7u8 {chord.fingerings.push(get_fingering(read_signed_byte(data, seek)));}
        chord.show = Some(read_bool(data, seek));
    }

    pub(crate) fn write_chord(&self, data: &mut  Vec<u8>, track: usize, measure: usize, voice: usize, beat: usize) {
        if let Some(c) = &self.tracks[track].measures[measure].voices[voice].beats[beat].effect.chord {
            write_bool(data, c.new_format == Some(true));
            if c.new_format == Some(true) {self.write_new_format_chord(data, c);}
            else {self.write_old_format_chord(data, c);}
        }
    }

    fn write_new_format_chord(&self, data: &mut Vec<u8>, chord: &Chord) {
        write_bool(data, chord.sharp == Some(true));
        write_placeholder_default(data, 3);
        //root
        if let Some(r) = &chord.root {write_i32(data, r.value.to_i32().unwrap());}
        else {write_i32(data, 0);}
        //chord type
        if let Some(t) = chord.kind {write_i32(data, from_chord_type(t).to_i32().unwrap());} 
        else {write_i32(data, 0);}
        //chord extension
        if let Some(e) = chord.extension {write_i32(data, from_chord_extension(e).to_i32().unwrap());}
        else {write_i32(data, 0);}
        //bass
        if let Some(b) = &chord.bass {write_i32(data, b.value.to_i32().unwrap());}
        else {write_i32(data, 0);}
        //tonality
        if let Some(t) = chord.tonality {write_i32(data, from_chord_alteration(t).to_i32().unwrap());}
        else {write_i32(data, 0);}
        //
        write_bool(data, chord.add == Some(true));
        write_byte_size_string(data, &chord.name);
        write_placeholder_default(data, 22 - chord.name.len());
        //fifth, ninth, eleventh
        if let Some(f) = chord.fifth    {write_i32(data, from_chord_alteration(f).to_i32().unwrap());}
        else {write_i32(data, 0);}
        if let Some(n) = chord.ninth    {write_i32(data, from_chord_alteration(n).to_i32().unwrap());}
        else {write_i32(data, 0);}
        if let Some(e) = chord.eleventh {write_i32(data, from_chord_alteration(e).to_i32().unwrap());}
        else {write_i32(data, 0);}
        //TODO:
    }
    fn write_old_format_chord(&self, data: &mut Vec<u8>, chord: &Chord) {
        write_int_byte_size_string(data, &chord.name);
        if let Some(ff) = chord.first_fret {write_i32(data, ff.to_i32().unwrap());}
        else {write_i32(data, 0);} //TODO: check
        //TODO: for fret in {write_i32(data, fret);}
    }
}

#[cfg(test)]
mod test {
    use crate::chord::PitchClass;

    #[test]
    fn test_pitch_1() {
        let p = PitchClass::from_note("D#".to_string());
        assert!(p.sharp, "D# is sharp? {}", true);
        assert_eq!(1, p.accidental);
    }
    #[test]
    fn test_pitch_2() {
        let p = PitchClass::from(4, Some(-1), None);
        assert_eq!(3, p.value);
        assert!(!p.sharp);
        assert_eq!("Eb", p.to_string(), "Note should be Eb");
    }
    #[test]
    fn test_pitch_3() {
        let p = PitchClass::from(4, Some(-1), Some(true));
        assert_eq!(3, p.value);
        assert_eq!("D#", p.to_string(), "Note should be D#");
    }
    #[test]
    fn test_pitch_4() {
        //let p = PitchClass::from(3, None, None);
        //TODO: assert_eq!("Eb", p.to_string(), "Note should be Eb"); //TODO: FIXME: error on the Python source
    }
    #[test]
    fn test_pitch_5() {
        let p = PitchClass::from(3, None, Some(true));
        assert_eq!("D#", p.to_string(), "Note should be D#");
    }
}
