use super::*;
use crate::error::{GpResult, ToPrimitiveGp};
use crate::{
    io::primitive::*,
    model::legacy::{beat::*, enums::*, key_signature::*, song::Song},
};

pub(super) fn read_notes(
    song: &mut Song,
    data: &[u8],
    seek: &mut usize,
    track_index: usize,
    beat: &mut Beat,
    duration: &Duration,
    note_effect: NoteEffect,
) -> GpResult<()> {
    let flags = read_byte(data, seek)?;
    for i in 0..song.tracks[track_index].strings.len() {
        if (flags & 1 << (7 - song.tracks[track_index].strings[i].0)) > 0 {
            let mut note = Note {
                effect: note_effect.clone(),
                ..Default::default()
            };
            if song.version.number < (5, 0, 0) {
                song.read_note(
                    data,
                    seek,
                    &mut note,
                    song.tracks[track_index].strings[i],
                    track_index,
                )?;
            } else {
                song.read_note_v5(
                    data,
                    seek,
                    &mut note,
                    song.tracks[track_index].strings[i],
                    track_index,
                )?;
            }
            beat.notes.push(note);
        }
        beat.duration = duration.clone();
    }
    Ok(())
}

pub(super) fn read_note(
    song: &mut Song,
    data: &[u8],
    seek: &mut usize,
    note: &mut Note,
    guitar_string: (i8, i8),
    track_index: usize,
) -> GpResult<()> {
    let flags = read_byte(data, seek)?;
    note.string = guitar_string.0;
    note.effect.ghost_note = (flags & 0x04) == 0x04;
    if (flags & 0x20) == 0x20 {
        note.kind = get_note_type(read_byte(data, seek)?);
    }
    if (flags & 0x01) == 0x01 {
        note.duration = Some(read_signed_byte(data, seek)?);
        note.tuplet = Some(read_signed_byte(data, seek)?);
    }
    if (flags & 0x10) == 0x10 {
        let v = read_signed_byte(data, seek)?;
        note.velocity = crate::model::legacy::effects::unpack_velocity(v.to_i16_gp("velocity")?);
    }
    if (flags & 0x20) == 0x20 {
        let fret = read_signed_byte(data, seek)?;
        let value = if note.kind == NoteType::Tie {
            song.get_tied_note_value(guitar_string.0, track_index)
        } else {
            fret.to_i16_gp("fret value")?
        };
        note.value = value.clamp(0, 99);
    }
    if (flags & 0x80) == 0x80 {
        note.effect.left_hand_finger = get_fingering(read_signed_byte(data, seek)?);
        note.effect.right_hand_finger = get_fingering(read_signed_byte(data, seek)?);
    }
    if (flags & 0x08) == 0x08 {
        if song.version.number == (3, 0, 0) {
            song.read_note_effects_v3(data, seek, note)?;
        } else if song.version.number.0 == 4 {
            song.read_note_effects_v4(data, seek, note)?;
        }
        if let Some(mut h) = note.effect.harmonic.take() {
            if h.kind == HarmonicType::Tapped {
                h.fret = Some(note.value.to_i8_gp("note value")? + 12);
            }
            note.effect.harmonic = Some(h);
        }
    }
    Ok(())
}

pub(super) fn read_note_v5(
    song: &mut Song,
    data: &[u8],
    seek: &mut usize,
    note: &mut Note,
    guitar_string: (i8, i8),
    track_index: usize,
) -> GpResult<()> {
    let flags = read_byte(data, seek)?;
    note.string = guitar_string.0;
    note.effect.heavy_accentuated_note = (flags & 0x02) == 0x02;
    note.effect.ghost_note = (flags & 0x04) == 0x04;
    note.effect.accentuated_note = (flags & 0x40) == 0x40;
    if (flags & 0x20) == 0x20 {
        note.kind = get_note_type(read_byte(data, seek)?);
    }
    if (flags & 0x10) == 0x10 {
        let v = read_signed_byte(data, seek)?;
        note.velocity = crate::model::legacy::effects::unpack_velocity(v.to_i16_gp("velocity")?);
    }
    if (flags & 0x20) == 0x20 {
        let fret = read_signed_byte(data, seek)?;
        let value = if note.kind == NoteType::Tie {
            song.get_tied_note_value(guitar_string.0, track_index)
        } else {
            fret.to_i16_gp("fret value")?
        };
        note.value = value.clamp(0, 99);
    }
    if (flags & 0x80) == 0x80 {
        note.effect.left_hand_finger = get_fingering(read_signed_byte(data, seek)?);
        note.effect.right_hand_finger = get_fingering(read_signed_byte(data, seek)?);
    }
    if (flags & 0x01) == 0x01 {
        note.duration_percent = read_double(data, seek)?.to_f32_gp("duration percent")?;
    }
    note.swap_accidentals = (read_byte(data, seek)? & 0x02) == 0x02;
    if (flags & 0x08) == 0x08 {
        song.read_note_effects_v4(data, seek, note)?;
    }
    Ok(())
}

pub(super) fn read_note_effects_v3(
    song: &Song,
    data: &[u8],
    seek: &mut usize,
    note: &mut Note,
) -> GpResult<()> {
    use crate::model::legacy::effects::SongEffectOps;
    let flags = read_byte(data, seek)?;
    note.effect.hammer = (flags & 0x02) == 0x02;
    note.effect.let_ring = (flags & 0x08) == 0x08;
    if (flags & 0x01) == 0x01 {
        note.effect.bend = song.read_bend_effect(data, seek)?;
    }
    if (flags & 0x10) == 0x10 {
        note.effect.grace = Some(song.read_grace_effect(data, seek)?);
    }
    if (flags & 0x04) == 0x04 {
        note.effect.slides.push(SlideType::ShiftSlideTo);
    }
    Ok(())
}

pub(super) fn read_note_effects_v4(
    song: &mut Song,
    data: &[u8],
    seek: &mut usize,
    note: &mut Note,
) -> GpResult<()> {
    use crate::model::legacy::effects::SongEffectOps;
    let flags1 = read_signed_byte(data, seek)?;
    let flags2 = read_signed_byte(data, seek)?;
    note.effect.hammer = (flags1 & 0x02) == 0x02;
    note.effect.let_ring = (flags1 & 0x08) == 0x08;
    note.effect.staccato = (flags2 & 0x01) == 0x01;
    note.effect.palm_mute = (flags2 & 0x02) == 0x02;
    note.effect.vibrato = (flags2 & 0x40) == 0x40 || note.effect.vibrato;
    if (flags1 & 0x01) == 0x01 {
        note.effect.bend = song.read_bend_effect(data, seek)?;
    }
    if (flags1 & 0x10) == 0x10 {
        if song.version.number >= (5, 0, 0) {
            note.effect.grace = Some(song.read_grace_effect_v5(data, seek)?);
        } else {
            note.effect.grace = Some(song.read_grace_effect(data, seek)?);
        }
    }
    if (flags2 & 0x04) == 0x04 {
        note.effect.tremolo_picking = Some(song.read_tremolo_picking(data, seek)?);
    }
    if (flags2 & 0x08) == 0x08 {
        if song.version.number >= (5, 0, 0) {
            note.effect.slides.extend(song.read_slides_v5(data, seek)?);
        } else {
            note.effect
                .slides
                .push(get_slide_type(read_signed_byte(data, seek)?)?);
        }
    }
    if (flags2 & 0x10) == 0x10 {
        if song.version.number >= (5, 0, 0) {
            note.effect.harmonic = Some(song.read_harmonic_v5(data, seek)?);
        } else {
            note.effect.harmonic = Some(song.read_harmonic(data, seek, note)?);
        }
    }
    if (flags2 & 0x20) == 0x20 {
        note.effect.trill = Some(song.read_trill(data, seek)?);
    }
    Ok(())
}

pub(super) fn get_tied_note_value(song: &Song, string_index: i8, track_index: usize) -> i16 {
    for m in (0usize..song.tracks[track_index].measures.len()).rev() {
        for v in (0usize..song.tracks[track_index].measures[m].voices.len()).rev() {
            for b in 0..song.tracks[track_index].measures[m].voices[v].beats.len() {
                if song.tracks[track_index].measures[m].voices[v].beats[b].status
                    != BeatStatus::Empty
                {
                    for n in 0..song.tracks[track_index].measures[m].voices[v].beats[b]
                        .notes
                        .len()
                    {
                        if song.tracks[track_index].measures[m].voices[v].beats[b].notes[n].string
                            == string_index
                        {
                            return song.tracks[track_index].measures[m].voices[v].beats[b].notes
                                [n]
                                .value;
                        }
                    }
                }
            }
        }
    }
    -1
}
