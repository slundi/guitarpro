use std::collections::HashMap;

use super::*;
use crate::convert::loss::LossReport;
use crate::error::{GpError, GpResult, ToPrimitiveGp};
use crate::model::legacy::enums::BeatStatus;
use crate::model::legacy::note::write::{on_track_string, written_notes};
use crate::{
    io::primitive::*,
    model::legacy::{enums::*, song::*},
};

pub trait SongHeaderOps {
    fn _add_measure_header(&mut self, header: MeasureHeader);
    fn read_clipboard(&mut self, data: &[u8], seek: &mut usize) -> GpResult<Option<Clipboard>>;
    fn read_measure_headers(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        measure_count: usize,
    ) -> GpResult<()>;
    fn read_measure_headers_v5(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        measure_count: usize,
        directions: &(HashMap<DirectionSign, i16>, HashMap<DirectionSign, i16>),
    ) -> GpResult<()>;
    /// Reads one header. A GP3/4 one's `repeat_alternative` is the stored
    /// ordinal, until [`SongHeaderOps::read_measure_headers`] turns it into
    /// the ending's passes ([`EndingWalks`]).
    fn read_measure_header(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        number: usize,
        previous: Option<MeasureHeader>,
    ) -> GpResult<(MeasureHeader, u8)>;
    fn read_measure_header_v5(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        number: usize,
        previous: Option<MeasureHeader>,
    ) -> GpResult<(MeasureHeader, u8)>;
    /// A GP3/4 ending's stored ordinal (its last pass): see
    /// [`EndingWalks`] for its passes.
    fn read_repeat_alternative(&mut self, data: &[u8], seek: &mut usize) -> GpResult<u8>;
    fn read_repeat_alternative_v5(&mut self, data: &[u8], seek: &mut usize) -> GpResult<u8>;
    fn read_directions(
        &self,
        data: &[u8],
        seek: &mut usize,
    ) -> GpResult<(HashMap<DirectionSign, i16>, HashMap<DirectionSign, i16>)>;
    fn write_measure_headers(&self, data: &mut Vec<u8>, version: &(u8, u8, u8)) -> GpResult<()>;
    fn write_measure_header(
        &self,
        data: &mut Vec<u8>,
        header: usize,
        previous: Option<usize>,
        version: &(u8, u8, u8),
    ) -> GpResult<()>;
    fn write_clipboard(&self, data: &mut Vec<u8>, version: &(u8, u8, u8)) -> GpResult<()>;
    fn write_directions(&self, data: &mut Vec<u8>) -> GpResult<()>;
}

impl SongHeaderOps for Song {
    fn _add_measure_header(&mut self, header: MeasureHeader) {
        self.measure_headers.push(header);
    }

    fn read_clipboard(&mut self, data: &[u8], seek: &mut usize) -> GpResult<Option<Clipboard>> {
        if !self.version.clipboard {
            return Ok(None);
        }
        let mut c = Clipboard {
            start_measure: read_int(data, seek)?,
            ..Default::default()
        };
        c.stop_measure = read_int(data, seek)?;
        c.start_track = read_int(data, seek)?;
        c.stop_track = read_int(data, seek)?;
        if self.version.number.0 == 5 {
            c.start_beat = read_int(data, seek)?;
            c.stop_beat = read_int(data, seek)?;
            c.sub_bar_copy = read_int(data, seek)? != 0;
        }
        println!("read_clipboard(): {:?}", c);
        Ok(Some(c))
    }

    fn read_measure_headers(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        measure_count: usize,
    ) -> GpResult<()> {
        let mut previous: Option<MeasureHeader> = None;
        for i in 1..measure_count + 1 {
            let r: (MeasureHeader, u8) = self.read_measure_header(data, seek, i, previous)?;
            previous = Some(r.0.clone());
            self.measure_headers.push(r.0);
        }
        // Each ending's passes from its ordinal, after the headers before.
        let mut walks = EndingWalks::default();
        for index in 0..self.measure_headers.len() {
            let header = &self.measure_headers[index];
            if header.repeat_alternative > 0 {
                let passes = walks.passes(
                    &self.measure_headers[..index],
                    header.repeat_alternative,
                    header.repeat_open,
                );
                self.measure_headers[index].repeat_alternative = passes;
            }
        }
        Ok(())
    }

    fn read_measure_headers_v5(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        measure_count: usize,
        directions: &(HashMap<DirectionSign, i16>, HashMap<DirectionSign, i16>),
    ) -> GpResult<()> {
        let mut previous: Option<MeasureHeader> = None;
        for i in 1..measure_count + 1 {
            let r: (MeasureHeader, u8) = self.read_measure_header_v5(data, seek, i, previous)?;
            previous = Some(r.0.clone());
            self.measure_headers.push(r.0);
        }
        // Each sign names the (1-based) measure it's in, if any; a measure
        // can have several.
        let mut signs: HashMap<usize, Vec<DirectionSign>> = HashMap::new();
        for (&sign, &measure) in directions.0.iter().chain(&directions.1) {
            if measure > 0 {
                let index = measure.to_usize_gp("direction measure index")? - 1;
                signs.entry(index).or_default().push(sign);
            }
        }
        // Added once per measure, so each is sorted once.
        for (index, signs) in signs {
            if let Some(header) = self.measure_headers.get_mut(index) {
                header.add_directions(signs);
            }
        }
        Ok(())
    }

    fn read_measure_header(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        number: usize,
        previous: Option<MeasureHeader>,
    ) -> GpResult<(MeasureHeader, u8)> {
        let flag = read_byte(data, seek)?;
        let mut mh = MeasureHeader {
            number: number.to_u16_gp("measure number")?,
            ..Default::default()
        };
        mh.start = 0;
        mh.triplet_feel = self.triplet_feel.clone();
        if (flag & 0x01) == 0x01 {
            mh.time_signature.numerator = read_signed_byte(data, seek)?;
        } else if number > 1 {
            mh.time_signature.numerator = previous
                .clone()
                .ok_or(GpError::MissingState {
                    field: "previous_measure_header",
                })?
                .time_signature
                .numerator;
        }
        if (flag & 0x02) == 0x02 {
            mh.time_signature.denominator.value =
                read_signed_byte(data, seek)?.to_u16_gp("time_signature denominator value")?;
        } else if number > 1 {
            mh.time_signature.denominator = previous
                .clone()
                .ok_or(GpError::MissingState {
                    field: "previous_measure_header",
                })?
                .time_signature
                .denominator;
        }

        mh.repeat_open = (flag & 0x04) == 0x04;
        if (flag & 0x08) == 0x08 {
            mh.repeat_close = read_signed_byte(data, seek)?;
        }
        if (flag & 0x10) == 0x10 {
            mh.repeat_alternative = if self.version.number.0 == 5 {
                self.read_repeat_alternative_v5(data, seek)?
            } else {
                self.read_repeat_alternative(data, seek)?
            };
        }
        if (flag & 0x20) == 0x20 {
            mh.marker = Some(read_marker(data, seek)?);
        }
        if (flag & 0x40) == 0x40 {
            mh.key_signature.key = read_signed_byte(data, seek)?;
            mh.key_signature.is_minor = read_signed_byte(data, seek)? != 0;
        } else if mh.number > 1 {
            mh.key_signature = previous
                .ok_or(GpError::MissingState {
                    field: "previous_measure_header",
                })?
                .key_signature;
        }
        mh.double_bar = (flag & 0x80) == 0x80;
        Ok((mh, flag))
    }

    fn read_measure_header_v5(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        number: usize,
        previous: Option<MeasureHeader>,
    ) -> GpResult<(MeasureHeader, u8)> {
        if previous.is_some() {
            *seek += 1;
        }
        let r = self.read_measure_header(data, seek, number, previous.clone())?;
        let mut mh = r.0;
        let flags = r.1;
        if mh.repeat_close > -1 {
            mh.repeat_close -= 1;
        }
        // GP5 stores beams whenever the time signature changes at all (either the
        // numerator OR the denominator bit is set), not only when both are set.
        // Matches the Python `guitarpro` reference and TuxGuitar.
        if (flags & 0x03) != 0 {
            for i in 0..4 {
                mh.time_signature.beams[i] = read_byte(data, seek)?;
            }
        } else {
            mh.time_signature.beams = previous
                .ok_or(GpError::MissingState {
                    field: "previous_measure_header",
                })?
                .time_signature
                .beams;
        };
        if (flags & 0x10) == 0 {
            *seek += 1;
        }
        mh.triplet_feel = get_triplet_feel(read_byte(data, seek)?);
        Ok((mh, flags))
    }

    fn read_repeat_alternative(&mut self, data: &[u8], seek: &mut usize) -> GpResult<u8> {
        read_byte(data, seek)
    }

    fn read_repeat_alternative_v5(&mut self, data: &[u8], seek: &mut usize) -> GpResult<u8> {
        read_byte(data, seek)
    }

    fn read_directions(
        &self,
        data: &[u8],
        seek: &mut usize,
    ) -> GpResult<(HashMap<DirectionSign, i16>, HashMap<DirectionSign, i16>)> {
        let mut signs: HashMap<DirectionSign, i16> = HashMap::with_capacity(4);
        let mut from_signs: HashMap<DirectionSign, i16> = HashMap::with_capacity(15);
        signs.insert(DirectionSign::Coda, read_short(data, seek)?);
        signs.insert(DirectionSign::DoubleCoda, read_short(data, seek)?);
        signs.insert(DirectionSign::Segno, read_short(data, seek)?);
        signs.insert(DirectionSign::SegnoSegno, read_short(data, seek)?);
        signs.insert(DirectionSign::Fine, read_short(data, seek)?);
        from_signs.insert(DirectionSign::DaCapo, read_short(data, seek)?);
        from_signs.insert(DirectionSign::DaCapoAlCoda, read_short(data, seek)?);
        from_signs.insert(DirectionSign::DaCapoAlDoubleCoda, read_short(data, seek)?);
        from_signs.insert(DirectionSign::DaCapoAlFine, read_short(data, seek)?);
        from_signs.insert(DirectionSign::DaSegno, read_short(data, seek)?);
        from_signs.insert(DirectionSign::DaSegnoAlCoda, read_short(data, seek)?);
        from_signs.insert(DirectionSign::DaSegnoAlDoubleCoda, read_short(data, seek)?);
        from_signs.insert(DirectionSign::DaSegnoAlFine, read_short(data, seek)?);
        from_signs.insert(DirectionSign::DaSegnoSegno, read_short(data, seek)?);
        from_signs.insert(DirectionSign::DaSegnoSegnoAlCoda, read_short(data, seek)?);
        from_signs.insert(
            DirectionSign::DaSegnoSegnoAlDoubleCoda,
            read_short(data, seek)?,
        );
        from_signs.insert(DirectionSign::DaSegnoSegnoAlFine, read_short(data, seek)?);
        from_signs.insert(DirectionSign::DaCoda, read_short(data, seek)?);
        from_signs.insert(DirectionSign::DaDoubleCoda, read_short(data, seek)?);
        Ok((signs, from_signs))
    }

    fn write_measure_headers(&self, data: &mut Vec<u8>, version: &(u8, u8, u8)) -> GpResult<()> {
        let mut previous: Option<usize> = None;
        for i in 0..self.measure_headers.len() {
            self.write_measure_header(data, i, previous, version)?;
            previous = Some(i);
        }
        Ok(())
    }

    fn write_measure_header(
        &self,
        data: &mut Vec<u8>,
        header: usize,
        previous: Option<usize>,
        version: &(u8, u8, u8),
    ) -> GpResult<()> {
        let mut flags: u8 = 0x00;
        if let Some(p) = previous {
            if self.measure_headers[header].time_signature.numerator
                != self.measure_headers[p].time_signature.numerator
            {
                flags |= 0x01;
            }
            if self.measure_headers[header]
                .time_signature
                .denominator
                .value
                != self.measure_headers[p].time_signature.denominator.value
            {
                flags |= 0x02;
            }
        } else {
            flags |= 0x01;
            flags |= 0x02;
        }
        if self.measure_headers[header].repeat_open {
            flags |= 0x04;
        }
        if self.measure_headers[header].repeat_close > -1 {
            flags |= 0x08;
        }
        if self.measure_headers[header].repeat_alternative > 0 {
            flags |= 0x10;
        }
        if self.measure_headers[header].marker.is_some() {
            flags |= 0x20;
        }
        if self.measure_headers[header].double_bar {
            flags |= 0x80;
        }
        if version.0 >= 4 {
            if previous.is_none() {
                flags |= 0x40;
            } else if let Some(p) = previous
                && self.measure_headers[header].key_signature
                    != self.measure_headers[p].key_signature
            {
                flags |= 0x40;
            }
        }
        if version.0 >= 5
            && let Some(p) = previous
        {
            if self.measure_headers[header].time_signature != self.measure_headers[p].time_signature
            {
                flags |= 0x03;
            }
            write_placeholder_default(data, 1);
        }
        write_byte(data, flags);
        if (flags & 0x01) == 0x01 {
            write_signed_byte(data, self.measure_headers[header].time_signature.numerator);
        }
        if (flags & 0x02) == 0x02 {
            write_signed_byte(
                data,
                self.measure_headers[header]
                    .time_signature
                    .denominator
                    .value
                    .to_i8_gp("denominator value")?,
            );
        }
        if (flags & 0x08) == 0x08 {
            write_signed_byte(
                data,
                if version.0 < 5 {
                    self.measure_headers[header].repeat_close
                } else {
                    self.measure_headers[header].repeat_close.saturating_add(1)
                },
            );
        }
        if (flags & 0x10) == 0x10 {
            if version.0 == 5 {
                write_byte(data, self.measure_headers[header].repeat_alternative);
            } else {
                write_byte(
                    data,
                    ending_ordinal(self.measure_headers[header].repeat_alternative),
                );
            }
        }
        if (flags & 0x20) == 0x20
            && let Some(marker) = &self.measure_headers[header].marker
        {
            write_int_size_string(data, &marker.title);
            write_color(data, marker.color);
        }
        if version.0 >= 4 && (flags & 0x40) == 0x40 {
            write_signed_byte(data, self.measure_headers[header].key_signature.key);
            write_signed_byte(
                data,
                i8::from(self.measure_headers[header].key_signature.is_minor),
            );
        }
        if version.0 >= 5 {
            if (flags & 0x03) != 0 {
                for i in 0..self.measure_headers[header].time_signature.beams.len() {
                    write_byte(data, self.measure_headers[header].time_signature.beams[i]);
                }
            }
            if (flags & 0x10) == 0 {
                write_placeholder_default(data, 1);
            }
            write_byte(
                data,
                from_triplet_feel(&self.measure_headers[header].triplet_feel),
            );
        }
        Ok(())
    }

    fn write_clipboard(&self, data: &mut Vec<u8>, version: &(u8, u8, u8)) -> GpResult<()> {
        if let Some(c) = &self.clipboard {
            write_i32(data, c.start_measure.to_i32_gp("clipboard start_measure")?);
            write_i32(data, c.stop_measure.to_i32_gp("clipboard stop_measure")?);
            write_i32(data, c.start_track.to_i32_gp("clipboard start_track")?);
            write_i32(data, c.stop_track.to_i32_gp("clipboard stop_track")?);
            if version.0 == 5 {
                write_i32(data, c.start_beat.to_i32_gp("clipboard start_beat")?);
                write_i32(data, c.stop_beat.to_i32_gp("clipboard stop_beat")?);
                write_i32(data, i32::from(c.sub_bar_copy));
            }
        }
        Ok(())
    }

    fn write_directions(&self, data: &mut Vec<u8>) -> GpResult<()> {
        let measures = direction_measures(&self.measure_headers);
        for sign in DirectionSign::ALL {
            let measure = match measures.get(&sign) {
                Some(&index) => (index + 1).to_i16_gp("direction measure index")?,
                None => -1,
            };
            write_i16(data, measure);
        }
        Ok(())
    }
}

/// GP3/4 endings' passes from their stored ordinals, one header after
/// another. Keeps, per measure in none, the earlier endings of the repeat
/// after it, so each measure is walked once over a song.
#[derive(Default)]
pub(crate) struct EndingWalks {
    walks: Vec<u16>,
}

impl EndingWalks {
    /// The passes of the ending a measure that `opens` a repeat or not
    /// stores as `ordinal`, after `headers` (the ones before it, endings
    /// already as passes). Called for each ending in order, after the
    /// headers before it are final.
    pub(crate) fn passes(&mut self, headers: &[MeasureHeader], ordinal: u8, opens: bool) -> u8 {
        let walks = &mut self.walks;
        // Guitar Pro encodes repeat alternatives as a bitmask in a single byte,
        // so `value` should stay in `1..=8`. Some exports emit larger numbers
        // (spec says "1st/2nd/…" but the file may carry a raw ordinal). Clamp
        // to 8 so `(1 << value) - 1` doesn't overflow a u16 and so the result
        // still fits `u8`.
        let value = u16::from(ordinal).min(8);
        // A measure opening a repeat starts it: no ending before it is its.
        //
        // The ending before this measure, not closed by a repeat there and
        // of this ordinal (its last pass): this restates it, continuing it
        // (a multi-measure ending). A lower ordinal isn't its.
        if value > 0
            && !opens
            && let Some(previous) = headers.last()
            && previous.repeat_alternative != 0
            && u8::BITS - previous.repeat_alternative.leading_zeros() == u32::from(value)
            && previous.repeat_close < 0
        {
            return previous.repeat_alternative;
        }
        // This repeat's earlier endings, back to its start
        // ([`earlier_endings`]), with each measure in none's walk on
        // cached: each measure is walked once over a file.
        let existing_alternative = if opens {
            0
        } else {
            // Walks past these headers were for other ones.
            walks.truncate(headers.len());
            for gap in walks.len()..headers.len() {
                let header = &headers[gap];
                let walk = if header.repeat_alternative == 0 && !header.repeat_open {
                    earlier_endings(headers, gap, true, walks)
                } else {
                    0
                };
                walks.push(walk);
            }
            earlier_endings(headers, headers.len(), false, walks)
        };
        // The passes up to `value` no earlier ending in this repeat has.
        // A restated earlier ordinal, or one after the repeat's close,
        // stays 0.
        (((1u16 << value) - 1) & !existing_alternative) as u8
    }
}

/// The ordinal GP3/4 store an ending's passes `mask` as: the last pass of
/// its first run (passes 1 and 2 → 2; 1 and 3 → 1, losing 3).
pub(crate) fn ending_ordinal(mask: u8) -> u8 {
    if mask == 0 {
        return 0;
    }
    // The first run's start, plus its length.
    let start = mask.trailing_zeros();
    (start + (mask >> start).trailing_ones()) as u8
}

impl Song {
    /// What writing this song as Guitar Pro `version` can't keep: for GP3/4,
    /// endings whose passes their single ordinal doesn't give back (as
    /// [`EndingWalks`] reads it back: passes 1 and 3
    /// before a 2nd ending) and every direction sign (they have none); for
    /// GP5, a direction sign's measures past the one it stores; and the
    /// notes of voices past the ones it holds (GP3/4: one, GP5: two).
    pub fn write_losses(&self, version: (u8, u8, u8)) -> LossReport {
        let mut report = LossReport::new();
        let voices = if version.0 >= 5 { 2 } else { 1 };
        let has_notes = |voice: &&crate::model::legacy::beat::Voice| {
            voice
                .beats
                .iter()
                .any(|beat| beat.status == BeatStatus::Normal && !beat.notes.is_empty())
        };
        for measure in self.tracks.iter().flat_map(|track| &track.measures) {
            let dropped = measure.voices.iter().skip(voices).filter(has_notes).count();
            let name = if voices == 1 {
                "Voice past the first in a measure (GP3/4 hold one)"
            } else {
                "Voice past the second in a measure (GP5 holds two)"
            };
            report.note_n(name, dropped);
            // All its notes in those: the measure is written empty.
            if dropped > 0 && !measure.voices.iter().take(voices).any(|v| has_notes(&v)) {
                report.note("Measure written empty (its notes only in voices it can't hold)");
            }
        }
        // Notes the writer leaves out: on none of their track's strings (a
        // converter's fallback for one it couldn't place), or on a string
        // an earlier note of the beat has (GP holds one per string).
        for track in &self.tracks {
            let beats = track
                .measures
                .iter()
                .flat_map(|measure| measure.voices.iter().take(voices))
                .flat_map(|voice| &voice.beats);
            for beat in beats {
                let off_strings = beat
                    .notes
                    .iter()
                    .filter(|note| !on_track_string(note.string, &track.strings))
                    .count();
                let kept = written_notes(beat, &track.strings).count();
                report.note_n("Note on no string of its track", off_strings);
                report.note_n(
                    "Note on a string another note of its beat has (first kept)",
                    beat.notes.len().saturating_sub(kept + off_strings),
                );
            }
        }
        if version.0 >= 5 {
            let kept = direction_measures(&self.measure_headers).len();
            let signs: usize = self
                .measure_headers
                .iter()
                .map(|header| header.directions.len())
                .sum();
            report.note_n(
                "Direction sign in another measure (GP5 keeps one per sign)",
                signs.saturating_sub(kept),
            );
            return report;
        }
        // The headers as a reader rebuilds them, each ending from its
        // ordinal after the ones before.
        let mut read_back: Vec<MeasureHeader> = Vec::new();
        let mut walks = EndingWalks::default();
        for header in &self.measure_headers {
            let mut repeat_alternative = 0;
            if header.repeat_alternative > 0 {
                let ordinal = ending_ordinal(header.repeat_alternative);
                repeat_alternative = walks.passes(&read_back, ordinal, header.repeat_open);
                if repeat_alternative != header.repeat_alternative {
                    report.note("Ending passes (GP3/4 store one ordinal per ending)");
                }
            }
            read_back.push(MeasureHeader {
                repeat_open: header.repeat_open,
                repeat_close: header.repeat_close,
                repeat_alternative,
                ..MeasureHeader::default()
            });
            for _ in &header.directions {
                report.note("Direction sign (GP3/4 have none)");
            }
        }
        report
    }
}

/// The one measure (0-based) GP5 stores each sign at, which has no room
/// for others: of a sign in several, the one playback goes by. A jump's
/// first (the first reached); the segno before the first jump to it; the
/// fine at or after where the first al Fine jump returns to; the coda
/// after the first To Coda. Without such a jump, a sign's first.
fn direction_measures(headers: &[MeasureHeader]) -> HashMap<DirectionSign, usize> {
    use DirectionSign as D;
    // Each sign's measures, in order: one pass over the headers.
    let mut by_sign: HashMap<DirectionSign, Vec<usize>> = HashMap::new();
    for (index, header) in headers.iter().enumerate() {
        for &sign in &header.directions {
            by_sign.entry(sign).or_default().push(index);
        }
    }
    let at = |sign: DirectionSign| -> &[usize] { by_sign.get(&sign).map_or(&[], Vec::as_slice) };
    let first_of =
        |signs: &[DirectionSign]| signs.iter().filter_map(|&s| at(s).first().copied()).min();
    let mut measures: HashMap<DirectionSign, usize> = DirectionSign::ALL
        .into_iter()
        .filter_map(|sign| at(sign).first().map(|&index| (sign, index)))
        .collect();
    // The last of `sign` at or before `limit`.
    let last_before = |sign, limit: Option<usize>| {
        limit.and_then(|limit| at(sign).iter().copied().rfind(|&i| i <= limit))
    };
    // The first of `sign` at or after `from`.
    let first_from = |sign, from: Option<usize>| {
        from.and_then(|from| at(sign).iter().copied().find(|&i| i >= from))
    };
    let segno_jumps = [
        D::DaSegno,
        D::DaSegnoAlCoda,
        D::DaSegnoAlDoubleCoda,
        D::DaSegnoAlFine,
    ];
    let segno_segno_jumps = [
        D::DaSegnoSegno,
        D::DaSegnoSegnoAlCoda,
        D::DaSegnoSegnoAlDoubleCoda,
        D::DaSegnoSegnoAlFine,
    ];
    let segno = last_before(D::Segno, first_of(&segno_jumps)).or(measures.get(&D::Segno).copied());
    let segno_segno = last_before(D::SegnoSegno, first_of(&segno_segno_jumps))
        .or(measures.get(&D::SegnoSegno).copied());
    let fine_jump = [D::DaCapoAlFine, D::DaSegnoAlFine, D::DaSegnoSegnoAlFine]
        .into_iter()
        .filter_map(|sign| at(sign).first().map(|&index| (index, sign)))
        .min();
    let fine_from = fine_jump.and_then(|(_, sign)| match sign {
        D::DaCapoAlFine => Some(0),
        D::DaSegnoAlFine => segno,
        _ => segno_segno,
    });
    let chosen = [
        (D::Segno, segno),
        (D::SegnoSegno, segno_segno),
        (D::Fine, first_from(D::Fine, fine_from)),
        (
            D::Coda,
            first_from(D::Coda, first_of(&[D::DaCoda]).map(|index| index + 1)),
        ),
        (
            D::DoubleCoda,
            first_from(
                D::DoubleCoda,
                first_of(&[D::DaDoubleCoda]).map(|index| index + 1),
            ),
        ),
    ];
    for (sign, index) in chosen {
        if let Some(index) = index {
            measures.insert(sign, index);
        }
    }
    measures
}

/// The endings of the repeat a measure after `headers[..end]` is in, as
/// passes: back to its start, a run of endings ending in one that doesn't
/// close the repeat, after ones that do, ends an earlier repeat (its last
/// ending), before this one's body; past a measure in none (`between`),
/// too, a run is this repeat's only when it closes it (sending it round
/// again). `walks` has, per measure in none before `end`, the endings the
/// walk on from it adds (this walked from it, `between`).
fn earlier_endings(headers: &[MeasureHeader], end: usize, between: bool, walks: &[u16]) -> u16 {
    let mut existing = 0u16;
    let mut run_start = true;
    let mut between = between;
    for (index, header) in headers[..end].iter().enumerate().rev() {
        if header.repeat_open {
            // An ending in the measure opening the repeat (`|: [1. :|`) is
            // its own when its run sends it round again; one that doesn't
            // is an earlier repeat's last ending, which this repeat starts
            // at.
            if header.repeat_alternative != 0 && run_closes(&headers[index..end]) {
                existing |= u16::from(header.repeat_alternative);
            }
            break;
        }
        if header.repeat_alternative == 0 {
            return existing | walks.get(index).copied().unwrap_or(0);
        }
        // Asked at most once per walk (only at its start), over the run
        // the walk then goes through itself.
        if run_start
            && header.repeat_close < 0
            && (between || run_closes(headers[..index].iter().rev()))
        {
            break;
        }
        run_start = false;
        between = false;
        existing |= u16::from(header.repeat_alternative);
    }
    existing
}

/// Whether the run of ending measures `run` starts with (in its order:
/// forward or back) has one closing a repeat.
fn run_closes<'a>(run: impl IntoIterator<Item = &'a MeasureHeader>) -> bool {
    run.into_iter()
        .take_while(|header| header.repeat_alternative != 0)
        .any(|header| header.repeat_close >= 0)
}
