use super::*;
use crate::error::{GpResult, ToPrimitiveGp};
use crate::{
    io::primitive::*,
    model::{effects::*, enums::*, key_signature::*, song::Song},
};

pub(super) fn write_beat_v3(song: &Song, data: &mut Vec<u8>, beat: &Beat) -> GpResult<()> {
    let mut flags = 0u8;
    if beat.duration.dotted {
        flags |= 0x01;
    }
    if beat.effect.is_chord() {
        flags |= 0x02;
    }
    if !beat.text.is_empty() {
        flags |= 0x04;
    }
    if !beat.effect.is_default() || beat.has_harmonic() || beat.has_vibrato() {
        flags |= 0x08;
    }
    if let Some(mtc) = &beat.effect.mix_table_change
        && mtc.is_just_wah()
    {
        flags |= 0x10;
    }
    if !beat.duration.is_default_tuplet() {
        flags |= 0x20;
    }
    if beat.status != BeatStatus::Normal {
        flags |= 0x40;
    }
    write_byte(data, flags);
    if (flags & 0x40) == 0x40 {
        write_byte(data, from_beat_status(&beat.status));
    }
    beat.duration.write_duration(data, flags)?;
    if (flags & 0x02) == 0x02 {
        use crate::model::chord::SongChordOps;
        song.write_chord(data, beat);
    }
    if (flags & 0x04) == 0x04 {
        write_int_byte_size_string(data, &beat.text);
    }
    if (flags & 0x08) == 0x08 {
        song.write_beat_effect_v3(data, beat)?;
    }
    if (flags & 0x10) == 0x10 {
        use crate::model::mix_table::SongMixTableOps;
        song.write_mix_table_change(data, &beat.effect.mix_table_change, &(3, 0, 0));
    }
    use crate::model::note::SongNoteOps;
    song.write_notes(data, beat, &Vec::new(), &(3, 0, 0))?;
    Ok(())
}

pub(super) fn write_beat(
    song: &Song,
    data: &mut Vec<u8>,
    beat: &Beat,
    strings: &[(i8, i8)],
    version: &(u8, u8, u8),
) -> GpResult<()> {
    let mut flags = 0u8;
    if beat.duration.dotted {
        flags |= 0x01;
    }
    if beat.effect.is_chord() {
        flags |= 0x02;
    }
    if !beat.text.is_empty() {
        flags |= 0x04;
    }
    if !beat.effect.is_default() {
        flags |= 0x08;
    }
    if let Some(mtc) = &beat.effect.mix_table_change
        && (mtc.is_just_wah() || version.0 > 4)
    {
        flags |= 0x10;
    }
    if !beat.duration.is_default_tuplet() {
        flags |= 0x20;
    }
    if beat.status != BeatStatus::Normal {
        flags |= 0x40;
    }
    write_byte(data, flags);
    if (flags & 0x40) == 0x40 {
        write_byte(data, from_beat_status(&beat.status));
    }
    beat.duration.write_duration(data, flags)?;
    if (flags & 0x02) == 0x02 {
        use crate::model::chord::SongChordOps;
        song.write_chord_v4(data, beat);
    }
    if (flags & 0x04) == 0x04 {
        write_int_byte_size_string(data, &beat.text);
    }
    if (flags & 0x08) == 0x08 {
        song.write_beat_effect_v4(data, beat, version)?;
    }
    if (flags & 0x10) == 0x10 {
        use crate::model::mix_table::SongMixTableOps;
        song.write_mix_table_change(data, &beat.effect.mix_table_change, version);
    }
    use crate::model::note::SongNoteOps;
    song.write_notes(data, beat, strings, version)?;
    if version.0 == 5 {
        let mut flags2 = 0i16;
        if beat.display.break_beam {
            flags2 |= 0x0001;
        }
        if beat.display.beam_direction == VoiceDirection::Down {
            flags2 |= 0x0002;
        }
        if beat.display.force_beam {
            flags2 |= 0x0004;
        }
        if beat.display.beam_direction == VoiceDirection::Up {
            flags2 |= 0x0008;
        }
        if beat.octave == Octave::Ottava {
            flags2 |= 0x0010;
        }
        if beat.octave == Octave::OttavaBassa {
            flags2 |= 0x0020;
        }
        if beat.octave == Octave::Quindicesima {
            flags2 |= 0x0040;
        }
        if beat.octave == Octave::QuindicesimaBassa {
            flags2 |= 0x0100;
        }
        if beat.display.tuplet_bracket == TupletBracket::Start {
            flags2 |= 0x0200;
        }
        if beat.display.tuplet_bracket == TupletBracket::End {
            flags2 |= 0x0400;
        }
        if beat.display.break_secondary != 0 {
            flags2 |= 0x0800;
        }
        if beat.display.break_secondary_tuplet {
            flags2 |= 0x1000;
        }
        if beat.display.force_bracket {
            flags2 |= 0x2000;
        }
        write_i16(data, flags2);
        if (flags2 & 0x0800) == 0x0800 {
            write_byte(data, beat.display.break_secondary);
        }
    }
    Ok(())
}

pub(super) fn write_beat_effect_v3(song: &Song, data: &mut Vec<u8>, beat: &Beat) -> GpResult<()> {
    let mut flags1: u8 = 0;
    if beat.has_vibrato() {
        flags1 |= 0x01;
    }
    if beat.effect.vibrato {
        flags1 |= 0x02;
    }
    if beat.has_harmonic() {
        for n in 0..beat.notes.len() {
            if let Some(h) = &beat.notes[n].effect.harmonic {
                if h.kind == HarmonicType::Natural {
                    flags1 |= 0x04;
                }
                if h.kind == HarmonicType::Artificial {
                    flags1 |= 0x08;
                }
            }
        }
    }
    if beat.effect.fade_in {
        flags1 |= 0x10;
    }
    if beat.effect.is_tremolo_bar() || beat.effect.is_slap_effect() {
        flags1 |= 0x20;
    }
    if beat.effect.stroke.direction != BeatStrokeDirection::None && beat.effect.stroke.value != 0 {
        flags1 |= 0x40;
    }
    write_byte(data, flags1);
    if (flags1 & 0x20) == 0x20 {
        write_byte(data, from_slap_effect(&beat.effect.slap_effect));
        song.write_tremolo_bar(data, &beat.effect.tremolo_bar)?;
    }
    if (flags1 & 0x40) == 0x40 {
        song.write_beat_stroke(data, &beat.effect.stroke, &(3, 0, 0))?;
    }
    Ok(())
}

pub(super) fn write_beat_effect_v4(
    song: &Song,
    data: &mut Vec<u8>,
    beat: &Beat,
    version: &(u8, u8, u8),
) -> GpResult<()> {
    let mut flags1: i8 = 0;
    if beat.effect.vibrato {
        flags1 |= 0x02;
    }
    if beat.effect.fade_in {
        flags1 |= 0x10;
    }
    if beat.effect.is_slap_effect() {
        flags1 |= 0x20;
    }
    if beat.effect.stroke.direction != BeatStrokeDirection::None && beat.effect.stroke.value != 0 {
        flags1 |= 0x40;
    }
    write_signed_byte(data, flags1);

    let mut flags2 = 0i8;
    if beat.effect.has_rasgueado {
        flags2 |= 0x01;
    }
    if beat.effect.has_pick_stroke() {
        flags2 |= 0x02;
    }
    if beat.effect.is_tremolo_bar() {
        flags2 |= 0x04;
    }
    write_signed_byte(data, flags2);

    if (flags1 & 0x20) == 0x20 {
        write_signed_byte(
            data,
            from_slap_effect(&beat.effect.slap_effect).to_i8_gp("slap effect")?,
        );
    }
    if (flags2 & 0x04) == 0x04 {
        use crate::model::effects::SongEffectOps;
        song.write_bend(data, &beat.effect.tremolo_bar)?;
    }
    if (flags1 & 0x40) == 0x40 {
        song.write_beat_stroke(data, &beat.effect.stroke, version)?;
    }
    if (flags2 & 0x02) == 0x02 {
        write_signed_byte(data, from_beat_stroke_direction(&beat.effect.pick_stroke));
    }
    Ok(())
}

pub(super) fn write_tremolo_bar(
    _song: &Song,
    data: &mut Vec<u8>,
    bar: &Option<BendEffect>,
) -> GpResult<()> {
    if let Some(b) = bar {
        write_i32(data, b.value.to_i32_gp("tremolo bar value")?);
    } else {
        write_i32(data, 0);
    }
    Ok(())
}

pub(super) fn write_beat_stroke(
    _song: &Song,
    data: &mut Vec<u8>,
    stroke: &BeatStroke,
    version: &(u8, u8, u8),
) -> GpResult<()> {
    let mut stroke = stroke.clone();
    if version.0 == 5 {
        stroke.swap_direction();
    }
    let mut stroke_down = 0i8;
    let mut stroke_up = 0i8;
    if stroke.direction == BeatStrokeDirection::Up {
        stroke_up = from_stroke_value(stroke.value.to_u8_gp("stroke value")?);
    } else if stroke.direction == BeatStrokeDirection::Down {
        stroke_down = from_stroke_value(stroke.value.to_u8_gp("stroke value")?);
    }
    write_signed_byte(data, stroke_down);
    write_signed_byte(data, stroke_up);
    Ok(())
}

pub(super) fn from_stroke_value(value: u8) -> i8 {
    if value == DURATION_HUNDRED_TWENTY_EIGHTH {
        1
    } else if value == DURATION_SIXTY_FOURTH {
        2
    } else if value == DURATION_THIRTY_SECOND {
        3
    } else if value == DURATION_SIXTEENTH {
        4
    } else if value == DURATION_EIGHTH {
        5
    } else if value == DURATION_QUARTER {
        6
    } else {
        1
    }
}
