use std::collections::HashMap;

use super::*;
use crate::error::{GpError, GpResult, ToPrimitiveGp};
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
        for s in &directions.0 {
            if s.1 > &-1 {
                self.measure_headers[s.1.to_usize_gp("direction measure index")? - 1].direction =
                    Some(s.0.clone());
            }
        }
        for s in &directions.1 {
            if s.1 > &-1 {
                self.measure_headers[s.1.to_usize_gp("direction measure index")? - 1].direction =
                    Some(s.0.clone());
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
        let value = read_byte(data, seek)?.to_u16_gp("repeat alternative value")?;
        let mut existing_alternative = 0u16;
        for i in (0..self.measure_headers.len()).rev() {
            if self.measure_headers[i].repeat_open {
                break;
            }
            existing_alternative |= self.measure_headers[i]
                .repeat_alternative
                .to_u16_gp("repeat_alternative")?;
        }
        (((1 << value) - 1) ^ existing_alternative).to_u8_gp("repeat alternative result")
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
                    self.measure_headers[header].repeat_close + 1
                },
            );
        }
        if (flags & 0x10) == 0x10 {
            if version.0 == 5 {
                write_byte(data, self.measure_headers[header].repeat_alternative);
            } else {
                let ra = self.measure_headers[header].repeat_alternative;
                let mut first_one = false;
                let mut out: u8 = 0;
                for i in 0u8..9 - ra.leading_zeros().to_u8_gp("leading zeros")? {
                    out = i;
                    if (ra as u16 & (1u16 << i)) > 0 {
                        first_one = true;
                    } else if first_one {
                        break;
                    }
                }
                write_byte(data, out);
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
        let mut map: HashMap<DirectionSign, i16> = HashMap::with_capacity(19);
        for i in 0..self.measure_headers.len() {
            if let Some(d) = &self.measure_headers[i].direction {
                map.insert(d.clone(), (i + 1).to_i16_gp("direction measure index")?);
            }
        }
        let order: Vec<DirectionSign> = vec![
            DirectionSign::Coda,
            DirectionSign::DoubleCoda,
            DirectionSign::Segno,
            DirectionSign::SegnoSegno,
            DirectionSign::Fine,
            DirectionSign::DaCapo,
            DirectionSign::DaCapoAlCoda,
            DirectionSign::DaCapoAlDoubleCoda,
            DirectionSign::DaCapoAlFine,
            DirectionSign::DaSegno,
            DirectionSign::DaSegnoAlCoda,
            DirectionSign::DaSegnoAlDoubleCoda,
            DirectionSign::DaSegnoAlFine,
            DirectionSign::DaSegnoSegno,
            DirectionSign::DaSegnoSegnoAlCoda,
            DirectionSign::DaSegnoSegnoAlDoubleCoda,
            DirectionSign::DaSegnoSegnoAlFine,
            DirectionSign::DaCoda,
            DirectionSign::DaDoubleCoda,
        ];
        for d in order {
            let x = map.get(&d);
            if let Some(dir) = x {
                write_i16(data, *dir);
            } else {
                write_i16(data, -1);
            }
        }
        Ok(())
    }
}
