use super::*;
use crate::error::{GpResult, ToPrimitiveGp};
use crate::{
    io::primitive::*,
    model::legacy::{effects::*, enums::*, key_signature::*, note::*, song::Song},
};

pub(super) fn read_beat(
    song: &mut Song,
    data: &[u8],
    seek: &mut usize,
    voice: &mut Voice,
    start: i64,
    track_index: usize,
) -> GpResult<i64> {
    let flags = read_byte(data, seek)?;
    let mut b = 0;
    let mut new_beat = true;
    if song.version.number < (5, 0, 0) {
        for i in (0usize..voice.beats.len()).rev() {
            if voice.beats[i].start == Some(start) {
                b = i;
                new_beat = false;
                break;
            }
        }
    }
    if new_beat {
        voice.beats.push(Beat {
            start: Some(start),
            ..Default::default()
        });
        b = voice.beats.len() - 1;
    }

    if (flags & 0x40) == 0x40 {
        voice.beats[b].status = get_beat_status(read_byte(data, seek)?);
    }
    let duration = read_duration(data, seek, flags)?;
    let mut note_effect = NoteEffect::default();
    if (flags & 0x02) == 0x02 {
        use crate::model::legacy::chord::SongChordOps;
        voice.beats[b].effect.chord = Some(
            song.read_chord(
                data,
                seek,
                song.tracks[track_index]
                    .strings
                    .len()
                    .to_u8_gp("string count")?,
            )?,
        );
    }
    if (flags & 0x04) == 0x04 {
        voice.beats[b].text = read_int_byte_size_string(data, seek)?;
    }
    if (flags & 0x08) == 0x08 {
        let chord = voice.beats[b].effect.chord.clone();
        if song.version.number.0 == 3 {
            voice.beats[b].effect = song.read_beat_effects_v3(data, seek, &mut note_effect)?;
        } else {
            voice.beats[b].effect = song.read_beat_effects_v4(data, seek)?;
        }
        voice.beats[b].effect.chord = chord;
    }
    if (flags & 0x10) == 0x10 {
        use crate::model::legacy::mix_table::SongMixTableOps;
        let mtc = song.read_mix_table_change(data, seek)?;
        voice.beats[b].effect.mix_table_change = Some(mtc);
    }
    use crate::model::legacy::note::SongNoteOps;
    song.read_notes(
        data,
        seek,
        track_index,
        &mut voice.beats[b],
        &duration,
        note_effect,
    )?;
    Ok(if voice.beats[b].status == BeatStatus::Empty {
        0
    } else {
        duration.time()?.to_i64_gp("beat duration time")?
    })
}

pub(super) fn read_beat_v5(
    song: &mut Song,
    data: &[u8],
    seek: &mut usize,
    voice: &mut Voice,
    start: &mut i64,
    track_index: usize,
) -> GpResult<i64> {
    let duration = read_beat(song, data, seek, voice, *start, track_index)?;
    let b = voice.beats.len() - 1;

    let flags2 = read_short(data, seek)?;
    if (flags2 & 0x0010) == 0x0010 {
        voice.beats[b].octave = Octave::Ottava;
    }
    if (flags2 & 0x0020) == 0x0020 {
        voice.beats[b].octave = Octave::OttavaBassa;
    }
    if (flags2 & 0x0040) == 0x0040 {
        voice.beats[b].octave = Octave::Quindicesima;
    }
    if (flags2 & 0x0100) == 0x0100 {
        voice.beats[b].octave = Octave::QuindicesimaBassa;
    }

    voice.beats[b].display.break_beam = (flags2 & 0x0001) == 0x0001;
    voice.beats[b].display.force_beam = (flags2 & 0x0004) == 0x0004;
    voice.beats[b].display.force_bracket = (flags2 & 0x2000) == 0x2000;
    voice.beats[b].display.break_secondary_tuplet = (flags2 & 0x1000) == 0x1000;
    if (flags2 & 0x0002) == 0x0002 {
        voice.beats[b].display.beam_direction = VoiceDirection::Down;
    }
    if (flags2 & 0x0008) == 0x0008 {
        voice.beats[b].display.beam_direction = VoiceDirection::Up;
    }
    if (flags2 & 0x0200) == 0x0200 {
        voice.beats[b].display.tuplet_bracket = TupletBracket::Start;
    }
    if (flags2 & 0x0400) == 0x0400 {
        voice.beats[b].display.tuplet_bracket = TupletBracket::End;
    }
    if (flags2 & 0x0800) == 0x0800 {
        voice.beats[b].display.break_secondary = read_byte(data, seek)?;
    }

    Ok(duration)
}

pub(super) fn read_beat_effects_v3(
    song: &Song,
    data: &[u8],
    seek: &mut usize,
    note_effect: &mut NoteEffect,
) -> GpResult<BeatEffects> {
    let mut be = BeatEffects::default();
    let flags = read_byte(data, seek)?;
    note_effect.vibrato = (flags & 0x01) == 0x01 || note_effect.vibrato;
    be.vibrato = (flags & 0x02) == 0x02 || be.vibrato;
    be.fade_in = (flags & 0x10) == 0x10;
    if (flags & 0x20) == 0x20 {
        be.slap_effect = get_slap_effect(read_byte(data, seek)?)?;
        if be.slap_effect == SlapEffect::None {
            be.tremolo_bar = Some(song.read_tremolo_bar(data, seek)?);
        } else {
            read_int(data, seek)?;
        }
    }
    if (flags & 0x40) == 0x40 {
        be.stroke = song.read_beat_stroke(data, seek)?;
    }
    if (flags & 0x04) == 0x04 {
        note_effect.harmonic = Some(HarmonicEffect::default());
    }
    if (flags & 0x08) == 0x08 {
        note_effect.harmonic = Some(HarmonicEffect {
            kind: HarmonicType::Artificial,
            ..Default::default()
        });
    }
    Ok(be)
}

pub(super) fn read_beat_effects_v4(
    song: &Song,
    data: &[u8],
    seek: &mut usize,
) -> GpResult<BeatEffects> {
    let mut be = BeatEffects::default();
    let flags1 = read_signed_byte(data, seek)?;
    let flags2 = read_signed_byte(data, seek)?;
    be.vibrato = (flags1 & 0x02) == 0x02 || be.vibrato;
    be.fade_in = (flags1 & 0x10) == 0x10;
    if (flags1 & 0x20) == 0x20 {
        let v = read_signed_byte(data, seek)?;
        be.slap_effect = if v < 0 {
            SlapEffect::None
        } else {
            get_slap_effect(v as u8)?
        };
    }
    if (flags2 & 0x04) == 0x04 {
        use crate::model::legacy::effects::SongEffectOps;
        be.tremolo_bar = song.read_bend_effect(data, seek)?;
    }
    if (flags1 & 0x40) == 0x40 {
        be.stroke = song.read_beat_stroke(data, seek)?;
    }
    be.has_rasgueado = (flags2 & 0x01) == 0x01;
    if (flags2 & 0x02) == 0x02 {
        be.pick_stroke = get_beat_stroke_direction(read_signed_byte(data, seek)?)?;
    }
    Ok(be)
}

pub(super) fn read_beat_stroke(song: &Song, data: &[u8], seek: &mut usize) -> GpResult<BeatStroke> {
    let mut bs = BeatStroke::default();
    let down = read_signed_byte(data, seek)?;
    let up = read_signed_byte(data, seek)?;
    if up > 0 {
        bs.direction = BeatStrokeDirection::Up;
        bs.value = stroke_value(song, up).to_u16_gp("stroke value")?;
    }
    if down > 0 {
        bs.direction = BeatStrokeDirection::Down;
        bs.value = stroke_value(song, down).to_u16_gp("stroke value")?;
    }
    if song.version.number >= (5, 0, 0) {
        bs.swap_direction();
    }
    Ok(bs)
}

pub(super) fn stroke_value(_song: &Song, value: i8) -> u8 {
    match value {
        1 => DURATION_HUNDRED_TWENTY_EIGHTH,
        2 => DURATION_SIXTY_FOURTH,
        3 => DURATION_THIRTY_SECOND,
        4 => DURATION_SIXTEENTH,
        5 => DURATION_EIGHTH,
        6 => DURATION_QUARTER,
        _ => DURATION_SIXTY_FOURTH,
    }
}

pub(super) fn read_tremolo_bar(
    _song: &Song,
    data: &[u8],
    seek: &mut usize,
) -> GpResult<BendEffect> {
    let mut be = BendEffect {
        kind: BendType::Dip,
        ..Default::default()
    };
    // Tremolo bar value is display-only; some editors write junk in the field
    // when the effect is unused. Truncate to `i16` instead of erroring.
    be.value = read_int(data, seek)? as i16;
    be.points.push(BendPoint {
        position: 0,
        value: 0,
        ..Default::default()
    });
    // Derived from `be.value`, which we now accept as-is; clamp to `i8` here
    // so a garbage tremolo value does not fail the whole load.
    let midpoint_value = (-f32::from(be.value) / GP_BEND_SEMITONE)
        .round()
        .clamp(i8::MIN as f32, i8::MAX as f32) as i8;
    be.points.push(BendPoint {
        position: BEND_EFFECT_MAX_POSITION / 2,
        value: midpoint_value,
        ..Default::default()
    });
    be.points.push(BendPoint {
        position: BEND_EFFECT_MAX_POSITION,
        value: 0,
        ..Default::default()
    });
    Ok(be)
}
