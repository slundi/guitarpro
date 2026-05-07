use super::*;
use crate::error::{GpError, GpResult, ToPrimitiveGp};
use crate::{
    io::primitive::*,
    model::{beat::*, effects::*, enums::*, key_signature::*, song::Song},
};

pub(super) fn write_notes(
    song: &Song,
    data: &mut Vec<u8>,
    beat: &Beat,
    strings: &[(i8, i8)],
    version: &(u8, u8, u8),
) -> GpResult<()> {
    let mut string_flags: u8 = 0;
    for i in 0..beat.notes.len() {
        string_flags |= 1 << (7 - beat.notes[i].string);
    }
    write_byte(data, string_flags);
    let mut notes = beat.notes.clone();
    notes.sort_by_key(|k| k.string);
    for note in &notes {
        if version.0 == 3 {
            song.write_note_v3(data, note)?;
        } else if version.0 == 4 {
            song.write_note_v4(data, note, strings, version)?;
        } else if version.0 == 5 {
            song.write_note_v5(data, note, strings, version)?;
        }
    }
    Ok(())
}

pub(super) fn write_note_v3(song: &Song, data: &mut Vec<u8>, note: &Note) -> GpResult<()> {
    let flags: u8 = pack_note_flags(song, note, &(3, 0, 0));
    write_byte(data, flags);
    if (flags & 0x20) == 0x20 {
        write_byte(data, from_note_type(&note.kind));
    }
    if (flags & 0x01) == 0x01 {
        write_signed_byte(
            data,
            note.duration.ok_or(GpError::MissingState {
                field: "note duration",
            })?,
        );
        write_signed_byte(
            data,
            note.tuplet.ok_or(GpError::MissingState {
                field: "note tuplet",
            })?,
        );
    }
    if (flags & 0x10) == 0x10 {
        write_signed_byte(data, crate::model::effects::pack_velocity(note.velocity)?);
    }
    if (flags & 0x20) == 0x20 {
        if note.kind != NoteType::Rest {
            write_signed_byte(data, note.value.to_i8_gp("note value")?);
        } else {
            write_signed_byte(data, 0);
        }
    }
    if (flags & 0x08) == 0x08 {
        song.write_note_effects_v3(data, note)?;
    }
    Ok(())
}

pub(super) fn write_note_v4(
    song: &Song,
    data: &mut Vec<u8>,
    note: &Note,
    strings: &[(i8, i8)],
    version: &(u8, u8, u8),
) -> GpResult<()> {
    let flags: u8 = pack_note_flags(song, note, version);
    write_byte(data, flags);
    if (flags & 0x20) == 0x20 {
        write_byte(data, from_note_type(&note.kind));
    }
    if (flags & 0x01) == 0x01 {
        write_signed_byte(
            data,
            note.duration.ok_or(GpError::MissingState {
                field: "note duration",
            })?,
        );
        write_signed_byte(
            data,
            note.tuplet.ok_or(GpError::MissingState {
                field: "note tuplet",
            })?,
        );
    }
    if (flags & 0x10) == 0x10 {
        write_signed_byte(data, crate::model::effects::pack_velocity(note.velocity)?);
    }
    if (flags & 0x20) == 0x20 {
        if note.kind != NoteType::Rest {
            write_signed_byte(data, note.value.to_i8_gp("note value")?);
        } else {
            write_signed_byte(data, 0);
        }
    }
    if (flags & 0x80) == 0x80 {
        write_signed_byte(data, from_fingering(&note.effect.left_hand_finger));
        write_signed_byte(data, from_fingering(&note.effect.right_hand_finger));
    }
    if (flags & 0x08) == 0x08 {
        if version.0 == 3 {
            song.write_note_effects_v3(data, note)?;
        } else {
            song.write_note_effects(data, note, strings, version)?;
        }
    }
    Ok(())
}

pub(super) fn write_note_v5(
    song: &Song,
    data: &mut Vec<u8>,
    note: &Note,
    strings: &[(i8, i8)],
    version: &(u8, u8, u8),
) -> GpResult<()> {
    let flags: u8 = pack_note_flags(song, note, version);
    write_byte(data, flags);
    if (flags & 0x20) == 0x20 {
        write_byte(data, from_note_type(&note.kind));
    }
    if (flags & 0x10) == 0x10 {
        write_signed_byte(data, crate::model::effects::pack_velocity(note.velocity)?);
    }
    if (flags & 0x20) == 0x20 {
        if note.kind != NoteType::Tie {
            write_signed_byte(data, note.value.to_i8_gp("note value")?);
        } else {
            write_signed_byte(data, 0);
        }
    }
    if (flags & 0x80) == 0x80 {
        write_signed_byte(data, from_fingering(&note.effect.left_hand_finger));
        write_signed_byte(data, from_fingering(&note.effect.right_hand_finger));
    }
    if (flags & 0x01) == 0x01 {
        write_f64(data, note.duration_percent.to_f64_gp("duration percent")?);
    }
    let mut flags2 = 0u8;
    if note.swap_accidentals {
        flags2 |= 0x02;
    }
    write_byte(data, flags2);
    if (flags & 0x08) == 0x08 {
        song.write_note_effects(data, note, strings, version)?;
    }
    Ok(())
}

pub(super) fn pack_note_flags(_song: &Song, note: &Note, version: &(u8, u8, u8)) -> u8 {
    let mut flags: u8 = 0u8;
    if note.duration.is_some() && note.tuplet.is_some() {
        flags |= 0x01;
    }
    if note.effect.heavy_accentuated_note {
        flags |= 0x02;
    }
    if note.effect.ghost_note {
        flags |= 0x04;
    }
    if !note.effect.is_default() {
        flags |= 0x08;
    }
    if note.velocity != DEFAULT_VELOCITY {
        flags |= 0x10;
    }
    flags |= 0x20;
    if version.0 > 3 {
        if note.effect.accentuated_note {
            flags |= 0x40;
        }
        if note.effect.is_fingering() {
            flags |= 0x80;
        }
    }
    if version.0 >= 5 && (note.duration_percent - 1.0).abs() > 1e-3 {
        flags |= 0x01;
    }
    flags
}

pub(super) fn write_note_effects_v3(song: &Song, data: &mut Vec<u8>, note: &Note) -> GpResult<()> {
    use crate::model::effects::SongEffectOps;
    let mut flags1 = 0u8;
    if note.effect.is_bend() {
        flags1 |= 0x01;
    }
    if note.effect.hammer {
        flags1 |= 0x02;
    }
    if note.effect.slides.contains(&SlideType::ShiftSlideTo)
        || note.effect.slides.contains(&SlideType::LegatoSlideTo)
    {
        flags1 |= 0x04;
    }
    if note.effect.let_ring {
        flags1 |= 0x08;
    }
    if note.effect.is_grace() {
        flags1 |= 0x10;
    }
    write_byte(data, flags1);
    if (flags1 & 0x01) == 0x01 {
        song.write_bend(data, &note.effect.bend)?;
    }
    if (flags1 & 0x10) == 0x10 {
        song.write_grace(data, &note.effect.grace)?;
    }
    Ok(())
}

pub(super) fn write_note_effects(
    song: &Song,
    data: &mut Vec<u8>,
    note: &Note,
    strings: &[(i8, i8)],
    version: &(u8, u8, u8),
) -> GpResult<()> {
    use crate::model::effects::SongEffectOps;
    let mut flags1 = 0i8;
    if note.effect.is_bend() {
        flags1 |= 0x01;
    }
    if note.effect.hammer {
        flags1 |= 0x02;
    }
    if note.effect.let_ring {
        flags1 |= 0x08;
    }
    if note.effect.is_grace() {
        flags1 |= 0x10;
    }
    write_signed_byte(data, flags1);

    let mut flags2 = 0i8;
    if note.effect.staccato {
        flags2 |= 0x01;
    }
    if note.effect.palm_mute {
        flags2 |= 0x02;
    }
    if note.effect.is_tremollo_picking() {
        flags2 |= 0x04;
    }
    if !note.effect.slides.is_empty() {
        flags2 |= 0x08;
    }
    if note.effect.is_harmonic() {
        flags2 |= 0x10;
    }
    if note.effect.is_trill() {
        flags2 |= 0x20;
    }
    if note.effect.vibrato {
        flags2 |= 0x40;
    }
    write_signed_byte(data, flags2);

    if (flags1 & 0x01) == 0x01 {
        song.write_bend(data, &note.effect.bend)?;
    }
    if (flags1 & 0x10) == 0x10 {
        if version.0 < 5 {
            song.write_grace(data, &note.effect.grace)?;
        } else {
            song.write_grace_v5(data, &note.effect.grace)?;
        }
    }
    if (flags2 & 0x04) == 0x04
        && let Some(tp) = &note.effect.tremolo_picking
    {
        let duration_val = tp.duration.value.to_u8_gp("tremolo picking duration")?;
        let encoded = match duration_val {
            DURATION_EIGHTH => 1,
            DURATION_SIXTEENTH => 2,
            DURATION_THIRTY_SECOND => 3,
            _ => {
                return Err(GpError::WriteError(format!(
                    "Invalid tremolo picking duration: {}",
                    duration_val
                )));
            }
        };
        write_signed_byte(data, encoded);
    }
    if (flags2 & 0x08) == 0x08 {
        if version.0 < 5 {
            write_signed_byte(data, from_slide_type(&note.effect.slides[0]));
        } else {
            song.write_slides_v5(data, &note.effect.slides);
        }
    }
    if (flags2 & 0x10) == 0x10 {
        if version.0 < 5 {
            song.write_harmonic(data, note, strings)?;
        } else {
            song.write_harmonic_v5(data, note, strings)?;
        }
    }
    if (flags2 & 0x20) == 0x20 {
        let t =
            note.effect.trill.as_ref().ok_or_else(|| {
                GpError::WriteError("Trill flag set but no trill data".to_string())
            })?;
        write_signed_byte(data, t.fret);
        let duration_val = t.duration.value.to_u8_gp("trill duration")?;
        let encoded = match duration_val {
            DURATION_SIXTEENTH => 1,
            DURATION_THIRTY_SECOND => 2,
            DURATION_SIXTY_FOURTH => 3,
            _ => {
                return Err(GpError::WriteError(format!(
                    "Invalid trill duration: {}",
                    duration_val
                )));
            }
        };
        write_signed_byte(data, encoded);
    }
    Ok(())
}
