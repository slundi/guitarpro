use super::*;
use crate::error::{GpResult, ToPrimitiveGp};
use crate::{
    io::primitive::*,
    model::legacy::{enums::*, song::*},
};

pub trait SongChordOps {
    fn read_chord(&self, data: &[u8], seek: &mut usize, string_count: u8) -> GpResult<Chord>;
    fn read_old_format_chord(
        &self,
        data: &[u8],
        seek: &mut usize,
        chord: &mut Chord,
    ) -> GpResult<()>;
    fn read_new_format_chord_v3(
        &self,
        data: &[u8],
        seek: &mut usize,
        chord: &mut Chord,
    ) -> GpResult<()>;
    fn read_new_format_chord_v4(
        &self,
        data: &[u8],
        seek: &mut usize,
        chord: &mut Chord,
    ) -> GpResult<()>;
    fn write_chord(&self, data: &mut Vec<u8>, beat: &crate::model::legacy::beat::Beat);
    fn write_new_format_chord(&self, data: &mut Vec<u8>, chord: &Chord);
    fn write_old_format_chord(&self, data: &mut Vec<u8>, chord: &Chord);
    fn write_chord_v4(&self, data: &mut Vec<u8>, beat: &crate::model::legacy::beat::Beat);
}

impl SongChordOps for Song {
    fn read_chord(&self, data: &[u8], seek: &mut usize, string_count: u8) -> GpResult<Chord> {
        let mut c = Chord {
            length: string_count,
            strings: Vec::new(),
            ..Default::default()
        };
        c.new_format = Some(read_bool(data, seek)?);
        if c.new_format == Some(true) {
            if self.version.number.0 == 3 {
                self.read_new_format_chord_v3(data, seek, &mut c)?;
            } else {
                self.read_new_format_chord_v4(data, seek, &mut c)?;
            }
        } else {
            if self.version.number.0 == 3 {
                read_byte(data, seek)?;
            }
            self.read_old_format_chord(data, seek, &mut c)?;
        }
        Ok(c)
    }

    fn read_old_format_chord(
        &self,
        data: &[u8],
        seek: &mut usize,
        chord: &mut Chord,
    ) -> GpResult<()> {
        chord.name = read_int_byte_size_string(data, seek)?;
        chord.first_fret = Some(read_int(data, seek)? as u8);
        if chord.first_fret.is_some() {
            for _ in 0u8..6u8 {
                chord.strings.push(read_int(data, seek)? as i8);
            }
        }
        Ok(())
    }

    fn read_new_format_chord_v3(
        &self,
        data: &[u8],
        seek: &mut usize,
        chord: &mut Chord,
    ) -> GpResult<()> {
        // GP3 uses the same "0x01"-marked chord layout as GP4 for the fields
        // that share names (root, kind, extension, fifth, ninth, eleventh,
        // barre_*), stored as single bytes — NOT as 4-byte ints. Only bass,
        // tonality, first_fret and per-string frets are ints. This matches
        // FILE-STRUCTURE-CHORD-DIAGRAMS.md and how TuxGuitar/Perlence's GP4
        // reader consumes GP3 clipboards. GP3-vs-GP4 delta: 6 strings (not 7),
        // 2 barres (not 5), and no trailing fingerings / show byte.
        chord.sharp = Some(read_bool(data, seek)?);
        *seek += 3;
        chord.root = Some(PitchClass::from(
            read_byte(data, seek)? as i8,
            None,
            chord.sharp,
        ));
        chord.kind = Some(get_chord_type(read_byte(data, seek)?));
        chord.extension = Some(get_chord_extension(read_byte(data, seek)?));
        // Some real-world GP3 exports pack these `int` fields with values that
        // don't fit `i8`/`u8` (unused-slot sentinels, editor-specific junk in
        // padding bytes). Truncate rather than fail — the chord diagram is only
        // display metadata, and Guitar Pro itself tolerates the noise.
        chord.bass = Some(PitchClass::from(
            read_int(data, seek)? as i8,
            None,
            chord.sharp,
        ));
        chord.tonality = Some(get_chord_alteration(read_int(data, seek)? as u8)?);
        chord.add = Some(read_bool(data, seek)?);
        chord.name = read_byte_size_string(data, seek, 22)?;
        chord.fifth = Some(get_chord_alteration(read_byte(data, seek)?)?);
        chord.ninth = Some(get_chord_alteration(read_byte(data, seek)?)?);
        chord.eleventh = Some(get_chord_alteration(read_byte(data, seek)?)?);
        chord.first_fret = Some(read_int(data, seek)? as u8);
        for _ in 0u8..6u8 {
            chord.strings.push(read_int(data, seek)? as i8);
        }
        let barre_count = read_byte(data, seek)?.to_usize_gp("chord barre count")?;
        let mut barre_frets: Vec<u8> = Vec::with_capacity(2);
        let mut barre_starts: Vec<u8> = Vec::with_capacity(2);
        let mut barre_ends: Vec<u8> = Vec::with_capacity(2);
        for _ in 0u8..2u8 {
            barre_frets.push(read_byte(data, seek)?);
        }
        for _ in 0u8..2u8 {
            barre_starts.push(read_byte(data, seek)?);
        }
        for _ in 0u8..2u8 {
            barre_ends.push(read_byte(data, seek)?);
        }
        for i in 0..barre_count.min(2) {
            chord.barres.push(Barre {
                fret: barre_frets[i] as i8,
                start: barre_starts[i] as i8,
                end: barre_ends[i] as i8,
            });
        }
        for _ in 0u8..7u8 {
            chord.omissions.push(read_bool(data, seek)?);
        }
        *seek += 1;
        Ok(())
    }

    fn read_new_format_chord_v4(
        &self,
        data: &[u8],
        seek: &mut usize,
        chord: &mut Chord,
    ) -> GpResult<()> {
        chord.sharp = Some(read_bool(data, seek)?);
        *seek += 3;
        chord.root = Some(PitchClass::from(
            read_byte(data, seek)? as i8,
            None,
            chord.sharp,
        ));
        chord.kind = Some(get_chord_type(read_byte(data, seek)?));
        chord.extension = Some(get_chord_extension(read_byte(data, seek)?));
        let i = read_int(data, seek)?;
        chord.bass = Some(PitchClass::from(i as i8, None, chord.sharp));
        chord.tonality = Some(get_chord_alteration(read_int(data, seek)? as u8)?);
        chord.add = Some(read_bool(data, seek)?);
        chord.name = read_byte_size_string(data, seek, 22)?;
        chord.fifth = Some(get_chord_alteration(read_byte(data, seek)?)?);
        chord.ninth = Some(get_chord_alteration(read_byte(data, seek)?)?);
        chord.eleventh = Some(get_chord_alteration(read_byte(data, seek)?)?);
        chord.first_fret = Some(read_int(data, seek)? as u8);
        for _ in 0u8..7u8 {
            chord.strings.push(read_int(data, seek)? as i8);
        }
        let barre_count = read_byte(data, seek)?.to_usize_gp("chord barre count")?;
        let mut barre_frets: Vec<u8> = Vec::with_capacity(5);
        let mut barre_starts: Vec<u8> = Vec::with_capacity(5);
        let mut barre_ends: Vec<u8> = Vec::with_capacity(5);
        for _ in 0u8..5u8 {
            barre_frets.push(read_byte(data, seek)?);
        }
        for _ in 0u8..5u8 {
            barre_starts.push(read_byte(data, seek)?);
        }
        for _ in 0u8..5u8 {
            barre_ends.push(read_byte(data, seek)?);
        }
        for i in 0..barre_count.min(5) {
            chord.barres.push(Barre {
                fret: barre_frets[i] as i8,
                start: barre_starts[i] as i8,
                end: barre_ends[i] as i8,
            });
        }
        for _ in 0u8..7u8 {
            chord.omissions.push(read_bool(data, seek)?);
        }
        *seek += 1;
        for _ in 0u8..7u8 {
            chord
                .fingerings
                .push(get_fingering(read_signed_byte(data, seek)?));
        }
        chord.show = Some(read_bool(data, seek)?);
        Ok(())
    }

    fn write_chord(&self, data: &mut Vec<u8>, beat: &crate::model::legacy::beat::Beat) {
        if let Some(c) = &beat.effect.chord {
            write_bool(data, c.new_format == Some(true));
            if c.new_format == Some(true) {
                self.write_new_format_chord(data, c);
            } else {
                self.write_old_format_chord(data, c);
            }
        }
    }

    fn write_new_format_chord(&self, data: &mut Vec<u8>, chord: &Chord) {
        // Symmetric with `read_new_format_chord_v3` (GP3): most fields are bytes,
        // only bass/tonality/first_fret/strings are 4-byte ints.
        write_bool(data, chord.sharp == Some(true));
        write_placeholder_default(data, 3);
        write_byte(
            data,
            chord.root.as_ref().map(|r| r.value as u8).unwrap_or(0),
        );
        write_byte(data, chord.kind.as_ref().map(from_chord_type).unwrap_or(0));
        write_byte(
            data,
            chord
                .extension
                .as_ref()
                .map(from_chord_extension)
                .unwrap_or(0),
        );
        if let Some(b) = &chord.bass {
            write_i32(data, b.value as i32);
        } else {
            write_i32(data, 0);
        }
        if let Some(t) = &chord.tonality {
            write_i32(data, from_chord_alteration(t) as i32);
        } else {
            write_i32(data, 0);
        }
        write_bool(data, chord.add == Some(true));
        write_byte_size_string(data, &chord.name);
        write_placeholder_default(data, 22 - chord.name.len());
        write_byte(
            data,
            chord.fifth.as_ref().map(from_chord_alteration).unwrap_or(0),
        );
        write_byte(
            data,
            chord.ninth.as_ref().map(from_chord_alteration).unwrap_or(0),
        );
        write_byte(
            data,
            chord
                .eleventh
                .as_ref()
                .map(from_chord_alteration)
                .unwrap_or(0),
        );
        if let Some(ff) = chord.first_fret {
            write_i32(data, ff as i32);
        } else {
            write_i32(data, 0);
        }
        for i in 0..6 {
            if i < chord.strings.len() {
                write_i32(data, chord.strings[i] as i32);
            } else {
                write_i32(data, -1);
            }
        }
        let mut barres: Vec<Barre> = Vec::with_capacity(2);
        for i in 0..2usize {
            if i < chord.barres.len() {
                barres.push(chord.barres[i].clone());
            } else {
                break;
            }
        }
        write_byte(data, barres.len() as u8);
        while barres.len() < 2 {
            barres.push(Barre {
                fret: 0,
                start: 0,
                end: 0,
            });
        }
        for b in barres.iter().take(2) {
            write_byte(data, b.fret as u8);
        }
        for b in barres.iter().take(2) {
            write_byte(data, b.start as u8);
        }
        for b in barres.iter().take(2) {
            write_byte(data, b.end as u8);
        }
        for i in 0..7usize {
            if i < chord.omissions.len() {
                write_bool(data, chord.omissions[i]);
            } else {
                write_bool(data, true);
            }
        }
        write_placeholder_default(data, 1);
    }

    fn write_old_format_chord(&self, data: &mut Vec<u8>, chord: &Chord) {
        write_int_byte_size_string(data, &chord.name);
        if let Some(ff) = chord.first_fret {
            write_i32(data, ff as i32);
        } else {
            write_i32(data, 0);
        }
        for i in 0..6 {
            if i < chord.strings.len() {
                write_i32(data, chord.strings[i] as i32);
            } else {
                write_i32(data, -1);
            }
        }
    }

    fn write_chord_v4(&self, data: &mut Vec<u8>, beat: &crate::model::legacy::beat::Beat) {
        if let Some(c) = &beat.effect.chord {
            write_signed_byte(data, 1);
            write_bool(data, c.sharp == Some(true));
            write_placeholder_default(data, 3);
            if let Some(r) = &c.root {
                write_byte(data, r.value as u8);
            } else {
                write_byte(data, 0);
            }
            if let Some(t) = &c.kind {
                write_byte(data, from_chord_type(t));
            } else {
                write_byte(data, 0);
            }
            if let Some(e) = &c.extension {
                write_byte(data, from_chord_extension(e));
            } else {
                write_byte(data, 0);
            }
            if let Some(b) = &c.bass {
                write_i32(data, b.value as i32);
            } else {
                write_i32(data, 0);
            }
            if let Some(t) = &c.tonality {
                write_i32(data, from_chord_alteration(t) as i32);
            } else {
                write_i32(data, 0);
            }
            write_bool(data, c.add == Some(true));
            let name_bytes = c.name.len().min(22);
            write_byte_size_string(data, &c.name);
            write_placeholder_default(data, 22 - name_bytes);
            if let Some(f) = &c.fifth {
                write_byte(data, from_chord_alteration(f));
            } else {
                write_byte(data, 0);
            }
            if let Some(n) = &c.ninth {
                write_byte(data, from_chord_alteration(n));
            } else {
                write_byte(data, 0);
            }
            if let Some(e) = &c.eleventh {
                write_byte(data, from_chord_alteration(e));
            } else {
                write_byte(data, 0);
            }
            if let Some(ff) = c.first_fret {
                write_i32(data, ff as i32);
            } else {
                write_i32(data, 0);
            }
            for i in 0..7 {
                if i < c.strings.len() {
                    write_i32(data, c.strings[i] as i32);
                } else {
                    write_i32(data, -1);
                }
            }
            let barre_count = c.barres.len().min(5);
            write_byte(data, barre_count as u8);
            let mut barres = c.barres.clone();
            while barres.len() < 5 {
                barres.push(Barre {
                    fret: 0,
                    start: 0,
                    end: 0,
                });
            }
            for b in barres.iter().take(5) {
                write_byte(data, b.fret as u8);
            }
            for b in barres.iter().take(5) {
                write_byte(data, b.start as u8);
            }
            for b in barres.iter().take(5) {
                write_byte(data, b.end as u8);
            }
            for i in 0..7usize {
                if i < c.omissions.len() {
                    write_bool(data, c.omissions[i]);
                } else {
                    write_bool(data, true);
                }
            }
            write_placeholder_default(data, 1);
            for i in 0..7 {
                if i < c.fingerings.len() {
                    write_signed_byte(data, from_fingering(&c.fingerings[i]));
                } else {
                    write_signed_byte(data, -2);
                }
            }
            write_bool(data, c.show == Some(true));
        }
    }
}
