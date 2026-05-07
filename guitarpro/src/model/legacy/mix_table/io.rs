use super::*;
use crate::error::{GpResult, ToPrimitiveGp};
use crate::io::primitive::*;
use crate::model::legacy::{rse::*, song::*};

pub trait SongMixTableOps {
    fn read_mix_table_change(&mut self, data: &[u8], seek: &mut usize) -> GpResult<MixTableChange>;
    fn read_mix_table_change_values(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        mtc: &mut MixTableChange,
    ) -> GpResult<()>;
    fn read_mix_table_change_durations(
        &self,
        data: &[u8],
        seek: &mut usize,
        mtc: &mut MixTableChange,
    ) -> GpResult<()>;
    fn read_mix_table_change_flags(
        &self,
        data: &[u8],
        seek: &mut usize,
        mtc: &mut MixTableChange,
    ) -> GpResult<i8>;
    fn read_wah_effect(&self, data: &[u8], seek: &mut usize, flags: i8) -> GpResult<WahEffect>;
    fn write_mix_table_change(
        &self,
        data: &mut Vec<u8>,
        mix_table_change: &Option<MixTableChange>,
        version: &(u8, u8, u8),
    );
    fn write_mix_table_change_values(
        &self,
        data: &mut Vec<u8>,
        mix_table_change: &MixTableChange,
        version: &(u8, u8, u8),
    );
    fn write_mix_table_change_durations(
        &self,
        data: &mut Vec<u8>,
        mix_table_change: &MixTableChange,
        version: &(u8, u8, u8),
    );
    fn write_mix_table_change_flags_v4(
        &self,
        data: &mut Vec<u8>,
        mix_table_change: &MixTableChange,
    );
    fn write_mix_table_change_flags_v5(
        &self,
        data: &mut Vec<u8>,
        mix_table_change: &MixTableChange,
    );
}

impl SongMixTableOps for Song {
    fn read_mix_table_change(&mut self, data: &[u8], seek: &mut usize) -> GpResult<MixTableChange> {
        let mut tc = MixTableChange::default();
        self.read_mix_table_change_values(data, seek, &mut tc)?;
        self.read_mix_table_change_durations(data, seek, &mut tc)?;
        if self.version.number >= (4, 0, 0) {
            let flags = self.read_mix_table_change_flags(data, seek, &mut tc)?;
            if self.version.number >= (5, 0, 0) {
                tc.wah = Some(self.read_wah_effect(data, seek, flags)?);
                self.read_rse_instrument_effect(data, seek, &mut tc.rse)?;
            }
        }
        Ok(tc)
    }

    fn read_mix_table_change_values(
        &mut self,
        data: &[u8],
        seek: &mut usize,
        mtc: &mut MixTableChange,
    ) -> GpResult<()> {
        let b = read_signed_byte(data, seek)?;
        if b >= 0 {
            mtc.instrument = Some(MixTableItem {
                value: b.to_u8_gp("mix table instrument")?,
                ..Default::default()
            });
        }
        if self.version.number.0 == 5 {
            mtc.rse = self.read_rse_instrument(data, seek)?;
        }
        if self.version.number == (5, 0, 0) {
            *seek += 1;
        }
        let b = read_signed_byte(data, seek)?;
        if b >= 0 {
            mtc.volume = Some(MixTableItem {
                value: b.to_u8_gp("mix table volume")?,
                ..Default::default()
            });
        }
        let b = read_signed_byte(data, seek)?;
        if b >= 0 {
            mtc.balance = Some(MixTableItem {
                value: b.to_u8_gp("mix table balance")?,
                ..Default::default()
            });
        }
        let b = read_signed_byte(data, seek)?;
        if b >= 0 {
            mtc.chorus = Some(MixTableItem {
                value: b.to_u8_gp("mix table chorus")?,
                ..Default::default()
            });
        }
        let b = read_signed_byte(data, seek)?;
        if b >= 0 {
            mtc.reverb = Some(MixTableItem {
                value: b.to_u8_gp("mix table reverb")?,
                ..Default::default()
            });
        }
        let b = read_signed_byte(data, seek)?;
        if b >= 0 {
            mtc.phaser = Some(MixTableItem {
                value: b.to_u8_gp("mix table phaser")?,
                ..Default::default()
            });
        }
        let b = read_signed_byte(data, seek)?;
        if b >= 0 {
            mtc.tremolo = Some(MixTableItem {
                value: b.to_u8_gp("mix table tremolo")?,
                ..Default::default()
            });
        }
        if self.version.number >= (5, 0, 0) {
            mtc.tempo_name = read_int_byte_size_string(data, seek)?;
        }
        let b = read_int(data, seek)?;
        if b >= 0 {
            mtc.tempo = Some(MixTableItem {
                value: b.clamp(0, 255) as u8,
                ..Default::default()
            });
        }
        Ok(())
    }

    fn read_mix_table_change_durations(
        &self,
        data: &[u8],
        seek: &mut usize,
        mtc: &mut MixTableChange,
    ) -> GpResult<()> {
        if let Some(ref mut item) = mtc.volume {
            item.duration = read_signed_byte(data, seek)?.max(0) as u8;
        }
        if let Some(ref mut item) = mtc.balance {
            item.duration = read_signed_byte(data, seek)?.max(0) as u8;
        }
        if let Some(ref mut item) = mtc.chorus {
            item.duration = read_signed_byte(data, seek)?.max(0) as u8;
        }
        if let Some(ref mut item) = mtc.reverb {
            item.duration = read_signed_byte(data, seek)?.max(0) as u8;
        }
        if let Some(ref mut item) = mtc.phaser {
            item.duration = read_signed_byte(data, seek)?.max(0) as u8;
        }
        if let Some(ref mut item) = mtc.tremolo {
            item.duration = read_signed_byte(data, seek)?.max(0) as u8;
        }
        if let Some(ref mut item) = mtc.tempo {
            item.duration = read_signed_byte(data, seek)?.max(0) as u8;
            mtc.hide_tempo = false;
            if self.version.number >= (5, 0, 0) {
                mtc.hide_tempo = read_bool(data, seek)?;
            }
        }
        Ok(())
    }

    fn read_mix_table_change_flags(
        &self,
        data: &[u8],
        seek: &mut usize,
        mtc: &mut MixTableChange,
    ) -> GpResult<i8> {
        let flags = read_signed_byte(data, seek)?;
        if let Some(mut e) = mtc.volume.take() {
            e.all_tracks = (flags & 0x01) == 0x01;
            mtc.volume = Some(e);
        }
        if let Some(mut e) = mtc.balance.take() {
            e.all_tracks = (flags & 0x01) == 0x01;
            mtc.balance = Some(e);
        }
        if let Some(mut e) = mtc.chorus.take() {
            e.all_tracks = (flags & 0x01) == 0x01;
            mtc.chorus = Some(e);
        }
        if let Some(mut e) = mtc.reverb.take() {
            e.all_tracks = (flags & 0x01) == 0x01;
            mtc.reverb = Some(e);
        }
        if let Some(mut e) = mtc.phaser.take() {
            e.all_tracks = (flags & 0x01) == 0x01;
            mtc.phaser = Some(e);
        }
        if let Some(mut e) = mtc.tremolo.take() {
            e.all_tracks = (flags & 0x01) == 0x01;
            mtc.tremolo = Some(e);
        }
        if self.version.number >= (5, 0, 0) {
            mtc.use_rse = (flags & 0x40) == 0x40;
        }
        Ok(flags)
    }

    fn read_wah_effect(&self, data: &[u8], seek: &mut usize, flags: i8) -> GpResult<WahEffect> {
        Ok(WahEffect {
            value: read_signed_byte(data, seek)?,
            display: (flags & -0x80) == -0x80,
        })
    }

    fn write_mix_table_change(
        &self,
        data: &mut Vec<u8>,
        mix_table_change: &Option<MixTableChange>,
        version: &(u8, u8, u8),
    ) {
        if let Some(mtc) = mix_table_change {
            self.write_mix_table_change_values(data, mtc, version);
            self.write_mix_table_change_durations(data, mtc, version);
            if version.0 == 4 {
                self.write_mix_table_change_flags_v4(data, mtc);
            }
            if version.0 == 5 {
                self.write_mix_table_change_flags_v5(data, mtc);
                if let Some(w) = &mtc.wah {
                    write_signed_byte(data, w.value);
                } else {
                    write_signed_byte(data, WAH_EFFECT_NONE);
                }
                if *version > (5, 0, 0) {
                    self.write_rse_instrument_effect(data, &mtc.rse);
                }
            }
        }
    }

    fn write_mix_table_change_values(
        &self,
        data: &mut Vec<u8>,
        mix_table_change: &MixTableChange,
        version: &(u8, u8, u8),
    ) {
        if let Some(i) = &mix_table_change.instrument {
            write_signed_byte(data, i.value as i8);
        } else {
            write_signed_byte(data, -1);
        }
        if version.0 >= 5 {
            self.write_rse_instrument(data, &mix_table_change.rse, version);
        }
        if version == &(5, 0, 0) {
            write_placeholder_default(data, 1);
        }
        if let Some(i) = &mix_table_change.volume {
            write_signed_byte(data, i.value as i8);
        } else {
            write_signed_byte(data, -1);
        }
        if let Some(i) = &mix_table_change.balance {
            write_signed_byte(data, i.value as i8);
        } else {
            write_signed_byte(data, -1);
        }
        if let Some(i) = &mix_table_change.chorus {
            write_signed_byte(data, i.value as i8);
        } else {
            write_signed_byte(data, -1);
        }
        if let Some(i) = &mix_table_change.reverb {
            write_signed_byte(data, i.value as i8);
        } else {
            write_signed_byte(data, -1);
        }
        if let Some(i) = &mix_table_change.phaser {
            write_signed_byte(data, i.value as i8);
        } else {
            write_signed_byte(data, -1);
        }
        if let Some(i) = &mix_table_change.tremolo {
            write_signed_byte(data, i.value as i8);
        } else {
            write_signed_byte(data, -1);
        }
        if version.0 >= 5 {
            write_int_byte_size_string(data, &mix_table_change.tempo_name);
        }
        if let Some(t) = &mix_table_change.tempo {
            write_i32(data, t.value as i32);
        } else {
            write_i32(data, -1);
        }
    }

    fn write_mix_table_change_durations(
        &self,
        data: &mut Vec<u8>,
        mix_table_change: &MixTableChange,
        version: &(u8, u8, u8),
    ) {
        if let Some(i) = &mix_table_change.volume {
            write_signed_byte(data, i.duration as i8);
        }
        if let Some(i) = &mix_table_change.balance {
            write_signed_byte(data, i.duration as i8);
        }
        if let Some(i) = &mix_table_change.chorus {
            write_signed_byte(data, i.duration as i8);
        }
        if let Some(i) = &mix_table_change.reverb {
            write_signed_byte(data, i.duration as i8);
        }
        if let Some(i) = &mix_table_change.phaser {
            write_signed_byte(data, i.duration as i8);
        }
        if let Some(i) = &mix_table_change.tremolo {
            write_signed_byte(data, i.duration as i8);
        }
        if let Some(i) = &mix_table_change.tempo {
            write_signed_byte(data, i.duration as i8);
            if version >= &(5, 0, 0) {
                write_bool(data, mix_table_change.hide_tempo);
            }
        }
    }

    fn write_mix_table_change_flags_v4(
        &self,
        data: &mut Vec<u8>,
        mix_table_change: &MixTableChange,
    ) {
        let mut flags = 0i8;
        if let Some(i) = &mix_table_change.volume
            && i.all_tracks
        {
            flags |= 0x01;
        }
        if let Some(i) = &mix_table_change.balance
            && i.all_tracks
        {
            flags |= 0x02;
        }
        if let Some(i) = &mix_table_change.chorus
            && i.all_tracks
        {
            flags |= 0x04;
        }
        if let Some(i) = &mix_table_change.reverb
            && i.all_tracks
        {
            flags |= 0x08;
        }
        if let Some(i) = &mix_table_change.phaser
            && i.all_tracks
        {
            flags |= 0x10;
        }
        if let Some(i) = &mix_table_change.tremolo
            && i.all_tracks
        {
            flags |= 0x20;
        }
        write_signed_byte(data, flags);
    }

    fn write_mix_table_change_flags_v5(
        &self,
        data: &mut Vec<u8>,
        mix_table_change: &MixTableChange,
    ) {
        let mut flags = 0u8;
        if let Some(i) = &mix_table_change.volume
            && i.all_tracks
        {
            flags |= 0x01;
        }
        if let Some(i) = &mix_table_change.balance
            && i.all_tracks
        {
            flags |= 0x02;
        }
        if let Some(i) = &mix_table_change.chorus
            && i.all_tracks
        {
            flags |= 0x04;
        }
        if let Some(i) = &mix_table_change.reverb
            && i.all_tracks
        {
            flags |= 0x08;
        }
        if let Some(i) = &mix_table_change.phaser
            && i.all_tracks
        {
            flags |= 0x10;
        }
        if let Some(i) = &mix_table_change.tremolo
            && i.all_tracks
        {
            flags |= 0x20;
        }
        if mix_table_change.use_rse {
            flags |= 0x40;
        }
        if let Some(w) = &mix_table_change.wah
            && w.display
        {
            flags |= 0x80;
        }
        write_byte(data, flags);
    }
}
