//! Playback-oriented intermediate representation.
//! Computed from Score; never serialized.

use std::ops::Range;

use crate::model::optimized::global::TrackId;

pub struct PlaybackScore {
    pub tracks: Vec<PlaybackTrack>,
    pub timeline: Vec<PlaybackMeasure>,
}

impl PlaybackScore {
    // pub fn to_midi(&self) -> Vec<u8> {
    //     let mut midi = MidiBuilder::new();

    //     for track in self.tracks {
    //         let ch = track.channel;
    //         for segment in &track.segments {
    //             let start_tick = playback.measure_start_tick(segment.measure_range.start);

    //             // program change au début du segment
    //             midi.program_change(ch, segment.program, start_tick);

    //             // control changes initiaux
    //             for (cc, val) in &segment.controllers {
    //                 midi.control_change(ch, *cc, *val, start_tick);
    //             }

    //             // notes
    //             for beat in &segment.beats {
    //                 let tick = start_tick + beat.tick_offset;
    //                 for note in &beat.notes {
    //                     if note.tie == Some(TieType::End) {
    //                         continue;
    //                     }
    //                     let duration = beat.sounding_duration_ticks(note);
    //                     midi.note_on(ch, note.pitch.to_midi(), beat.velocity(), tick);
    //                     midi.note_off(ch, note.pitch.to_midi(), tick + duration);
    //                 }
    //             }
    //         }
    //     }

    //     midi.build()
    // }
}

pub struct PlaybackTrack {
    pub source: PlaybackSource,
    pub channel: u8, // assigned MIDI channel
    pub segments: Vec<PlaybackSegment>,
}

impl PlaybackTrack {
    // /// Build a playback track from one or more score tracks (merged view).
    // /// Splits into segments at each EffectEvent that changes the MIDI program.
    // pub fn from_score_track(
    //     score: &Score,
    //     track_ids: &[TrackId], // 1 or many if merged
    // ) -> Self {
    //     let mut segments = vec![];
    //     let mut current_effect = EffectState::default();
    //     let mut segment_start = 0u16;
    //     let mut current_beats = vec![];

    //     for measure_idx in 0..score.timeline.len() as u16 {
    //         for track_id in track_ids {
    //             let track = &score.tracks[*track_id];
    //             let Some(measure) = track.measures.get(&measure_idx) else {
    //                 continue;
    //             };

    //             for beat in measure.all_beats() {
    //                 // detect effect change
    //                 if let Some(ev) = beat.effect_event() {
    //                     let new_effect = ev.to_state();
    //                     if new_effect.channel != current_effect.channel {
    //                         // close current segment
    //                         segments.push(PlaybackSegment {
    //                             measure_range: segment_start..measure_idx,
    //                             program: current_effect.to_midi_program(),
    //                             controllers: current_effect.to_midi_controllers(),
    //                             beats: std::mem::take(&mut current_beats),
    //                         });
    //                         segment_start = measure_idx;
    //                         current_effect = new_effect;
    //                     }
    //                 }
    //                 current_beats.push(PlaybackBeat::from(beat));
    //             }
    //         }
    //     }
    //     last segment
    //     segments.push(PlaybackSegment {
    //         measure_range: segment_start..score.timeline.len() as u16,
    //         program: current_effect.to_midi_program(),
    //         controllers: current_effect.to_midi_controllers(),
    //         beats: current_beats,
    //     });

    //     Self {
    //         source: PlaybackSource::Merged(track_ids.to_vec()),
    //         channel: 0,
    //         segments,
    //     }
    // }
}

pub enum PlaybackSource {
    Single(TrackId),
    Merged(Vec<TrackId>),
}

/// One segment per contiguous block of measures sharing the same MIDI program.
/// A new segment is created at each EffectEvent that changes EffectChannel.
pub struct PlaybackSegment {
    pub measure_range: Range<u16>,  // half-open: start..end measure indices
    pub program: u8,                // MIDI program number (GM instrument)
    pub controllers: Vec<(u8, u8)>, // (CC number, value) at segment start
    pub beats: Vec<PlaybackBeat>,
}

pub struct PlaybackMeasure {
    pub start_tick: u32,
    pub duration_ticks: u32,
}

pub struct PlaybackBeat {
    pub tick_offset: u32,
    pub notes: Vec<PlaybackNote>,
}

pub struct PlaybackNote {
    pub midi_pitch: u8,
    pub sounding_ticks: u32, // follows tie chain
    pub velocity: u8,
}
